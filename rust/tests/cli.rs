use std::{fs, path::Path, process::Command};

use tempfile::TempDir;

fn write_fixture(home: &Path) {
    let settings = home.join("Library/Application Support/JetBrains/IntelliJIdea2026.2/options");
    fs::create_dir_all(&settings).unwrap();

    let alpha = home.join("projects/alpha");
    let beta = home.join("projects/beta");
    let missing = home.join("projects/missing");
    fs::create_dir_all(alpha.join(".idea")).unwrap();
    fs::write(alpha.join(".idea/.name"), "Alpha").unwrap();
    fs::create_dir_all(&beta).unwrap();

    fs::write(
        settings.join("recentProjects.xml"),
        format!(
            r#"<application><component name="RecentProjectsManager"><option name="additionalInfo"><map>
            <entry key="{}"><value><RecentProjectMetaInfo><option name="projectOpenTimestamp" value="100" /></RecentProjectMetaInfo></value></entry>
            <entry key="{}"><value><RecentProjectMetaInfo><option name="projectOpenTimestamp" value="200" /></RecentProjectMetaInfo></value></entry>
            <entry key="{}"><value><RecentProjectMetaInfo><option name="projectOpenTimestamp" value="300" /></RecentProjectMetaInfo></value></entry>
            </map></option></component></application>"#,
            alpha.display(),
            beta.display(),
            missing.display()
        ),
    )
    .unwrap();

    let macos = home.join("Applications/IntelliJ IDEA.app/Contents/MacOS");
    fs::create_dir_all(&macos).unwrap();
    fs::write(macos.join("idea"), b"#!/bin/sh\n").unwrap();
}

fn run(home: &Path, config: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_jb"))
        .env("HOME", home)
        .env("JB_CONFIG", config)
        .env("JB_BINARIES", home.join("empty-bin"))
        .args(args)
        .output()
        .unwrap()
}

fn run_in(cwd: &Path, home: &Path, config: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_jb"))
        .current_dir(cwd)
        .env("HOME", home)
        .env("JB_CONFIG", config)
        .env("JB_BINARIES", home.join("empty-bin"))
        .args(args)
        .output()
        .unwrap()
}

fn search(home: &Path, config: &Path, filter: &str, extra: &[&str]) -> serde_json::Value {
    let mut args = vec!["search", "--filter", filter];
    args.extend_from_slice(extra);
    let output = run(home, config, &args);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn search_hides_stale_and_sorts_by_recency() {
    let temp = TempDir::new().unwrap();
    let home = temp.path();
    write_fixture(home);
    let config = home.join("config.toml");

    let json = search(home, &config, "", &[]);
    let titles = json["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["title"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(titles, vec!["beta", "Alpha"]);
}

#[test]
fn show_stale_includes_missing_projects() {
    let temp = TempDir::new().unwrap();
    let home = temp.path();
    write_fixture(home);
    let config = home.join("config.toml");

    let json = search(home, &config, "", &["--show-stale"]);
    assert_eq!(json["items"].as_array().unwrap().len(), 3);
    assert!(
        json["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["title"] == "missing")
    );
}

#[test]
fn pin_moves_project_to_the_top() {
    let temp = TempDir::new().unwrap();
    let home = temp.path();
    write_fixture(home);
    let config = home.join("config.toml");
    let alpha = home.join("projects/alpha");

    let output = run(home, &config, &["pin", alpha.to_str().unwrap()]);
    assert!(output.status.success());

    let json = search(home, &config, "", &[]);
    assert_eq!(json["items"][0]["title"], "★ Alpha");
    assert_eq!(json["items"][0]["variables"]["jb_pinned"], "1");
}

#[test]
fn product_prefix_scopes_results() {
    let temp = TempDir::new().unwrap();
    let home = temp.path();
    write_fixture(home);
    let config = home.join("config.toml");

    let json = search(home, &config, "idea alpha", &[]);
    assert_eq!(json["items"].as_array().unwrap().len(), 1);
    assert_eq!(json["items"][0]["title"], "Alpha");
}

#[test]
fn global_search_offers_updates_without_hiding_matching_projects() {
    let temp = TempDir::new().unwrap();
    let home = temp.path();
    write_fixture(home);
    fs::write(home.join("projects/alpha/.idea/.name"), "update").unwrap();
    let config = home.join("config.toml");

    for (filter, version) in [
        ("update", "latest"),
        (" UPDATE  latest ", "latest"),
        ("update v0.2.0", "v0.2.0"),
        ("update v0.3.0-beta.1", "v0.3.0-beta.1"),
    ] {
        let json = search(home, &config, filter, &[]);
        let action = &json["items"][0];
        assert_eq!(action["variables"]["jb_action"], "update");
        assert_eq!(action["arg"], version);
        assert!(action.get("mods").is_none());
    }
    let json = search(home, &config, "update", &[]);
    assert_eq!(json["items"][1]["title"], "update");
    assert_eq!(json["items"][1]["variables"]["jb_product"], "idea");
    for (filter, extra) in [
        ("update", vec!["--product", "idea"]),
        ("update", vec!["--worktrees"]),
        ("idea update", vec![]),
        ("update demo", vec![]),
        ("update v0.2.0 extra", vec![]),
        ("update v0.2.0;touch injected", vec![]),
    ] {
        let json = search(home, &config, filter, &extra);
        assert!(
            json["items"]
                .as_array()
                .unwrap()
                .iter()
                .all(|item| { item["variables"]["jb_action"] != "update" })
        );
    }
    assert!(!home.join("injected").exists());
    assert!(!home.join("Library/Application Support/Alfred").exists());
}

#[test]
fn alfred_update_action_runs_in_background_and_reports_success_or_failure() {
    use std::os::unix::fs::PermissionsExt;

    let temp = TempDir::new().unwrap();
    let home = temp.path();
    let config = home.join("config.toml");
    let workflow = home.join("exported 'workflow' 工作流");
    let output = run(home, &config, &["export", workflow.to_str().unwrap()]);
    assert!(output.status.success());
    let value = plist::Value::from_file(workflow.join("info.plist")).unwrap();
    let root = value.as_dictionary().unwrap();
    let objects = root["objects"].as_array().unwrap();
    let search_object = objects
        .iter()
        .map(|value| value.as_dictionary().unwrap())
        .find(|object| {
            object["config"]
                .as_dictionary()
                .unwrap()
                .get("keyword")
                .and_then(plist::Value::as_string)
                == Some("jb")
        })
        .unwrap();
    let search_uid = search_object["uid"].as_string().unwrap();
    let target = root["connections"].as_dictionary().unwrap()[search_uid]
        .as_array()
        .unwrap()[0]
        .as_dictionary()
        .unwrap()["destinationuid"]
        .as_string()
        .unwrap();
    let action = objects
        .iter()
        .map(|value| value.as_dictionary().unwrap())
        .find(|object| object["uid"].as_string() == Some(target))
        .unwrap();
    let script = action["config"].as_dictionary().unwrap()["script"]
        .as_string()
        .unwrap();

    let mocks = home.join("mock commands");
    fs::create_dir_all(&mocks).unwrap();
    for (path, script) in [
        (
            workflow.join("bin/jb"),
            "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"$TEST_ARGS\"\nexit \"$TEST_STATUS\"\n",
        ),
        (
            mocks.join("osascript"),
            "#!/bin/sh\nprintf '%s\\n' \"$@\" >> \"$TEST_NOTIFICATIONS\"\nexit 0\n",
        ),
    ] {
        fs::write(&path, script).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    }
    for (status, version, message) in [
        ("0", "latest", "completed"),
        ("0", "v0.3.0-beta.1", "completed"),
        ("1", "latest", "failed"),
    ] {
        let json = search(home, &config, &format!("update {version}"), &[]);
        let item = &json["items"][0];
        let notifications = home.join("notifications");
        fs::write(&notifications, "").unwrap();
        let args = home.join("args");
        let output = Command::new("/bin/bash")
            .args(["-c", script, "alfred-action"])
            .arg(item["arg"].as_str().unwrap())
            .env(
                "jb_action",
                item["variables"]["jb_action"].as_str().unwrap(),
            )
            .env("PATH", format!("{}:/usr/bin:/bin", mocks.display()))
            .env("TEST_ARGS", &args)
            .env("TEST_NOTIFICATIONS", &notifications)
            .env("TEST_STATUS", status)
            .current_dir(&workflow)
            .output()
            .unwrap();
        assert_eq!(output.status.success(), status == "0");
        assert_eq!(
            fs::read_to_string(&args).unwrap(),
            format!("update\n{version}\n")
        );
        let notifications = fs::read_to_string(&notifications).unwrap();
        assert!(notifications.contains("Downloading and installing"));
        assert!(notifications.contains(message));
    }
}

#[test]
fn config_prints_effective_settings() {
    let temp = TempDir::new().unwrap();
    let home = temp.path();
    write_fixture(home);
    let config = home.join("config.toml");

    let output = run(home, &config, &["config"]);
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["show_stale"], false);
    assert_eq!(json["sort"], "recent");
    assert_eq!(json["terminal"], "Terminal");
    assert!(
        json["settings_roots"]
            .as_array()
            .unwrap()
            .iter()
            .any(|path| path.as_str().unwrap().ends_with("JetBrains"))
    );
}

#[test]
fn alfred_user_config_env_overrides() {
    let temp = TempDir::new().unwrap();
    let home = temp.path();
    write_fixture(home);
    let config = home.join("config.toml");

    let output = Command::new(env!("CARGO_BIN_EXE_jb"))
        .env("HOME", home)
        .env("JB_CONFIG", &config)
        .env("jb_terminal", "Ghostty")
        .env("jb_show_stale", "1")
        .env("jb_sort", "name")
        .env("jb_ignore", "*.foo\n*.bar")
        .args(["config"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["terminal"], "Ghostty");
    assert_eq!(json["show_stale"], true);
    assert_eq!(json["sort"], "name");
    let ignore = json["ignore"].as_array().unwrap();
    assert!(ignore.contains(&serde_json::Value::String("*.foo".to_string())));
    assert!(ignore.contains(&serde_json::Value::String("*.bar".to_string())));
}

#[test]
fn doctor_reports_settings_and_projects() {
    let temp = TempDir::new().unwrap();
    let home = temp.path();
    write_fixture(home);
    let config = home.join("config.toml");

    let output = run(home, &config, &["doctor"]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("settings roots:"));
    assert!(text.contains("settings dirs:"));
    assert!(text.contains("projects:"));
    assert!(text.contains("ignore patterns:"));
    assert!(text.contains("forgotten:"));
    assert!(text.contains("terminal: Terminal"));
}

#[test]
fn forget_and_remember_project() {
    let temp = TempDir::new().unwrap();
    let home = temp.path();
    write_fixture(home);
    let config = home.join("config.toml");
    let alpha = home.join("projects/alpha");

    let output = run(home, &config, &["forget", alpha.to_str().unwrap()]);
    assert!(output.status.success());

    let json = search(home, &config, "", &[]);
    assert!(
        !json["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["title"] == "Alpha")
    );

    let output = run(home, &config, &["remember", alpha.to_str().unwrap()]);
    assert!(output.status.success());

    let json = search(home, &config, "", &[]);
    assert!(
        json["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["title"] == "Alpha")
    );
}

#[test]
fn config_ignore_hides_projects() {
    let temp = TempDir::new().unwrap();
    let home = temp.path();
    write_fixture(home);
    let config = home.join("config.toml");
    fs::write(&config, "ignore = [\"*beta*\"]\n").unwrap();

    let json = search(home, &config, "", &[]);
    let titles = json["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["title"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert!(titles.contains(&"Alpha"));
    assert!(!titles.contains(&"beta"));
}

#[test]
fn install_writes_workflow_files() {
    let temp = TempDir::new().unwrap();
    let home = temp.path();
    write_fixture(home);
    let config = home.join("config.toml");

    let alfred = home.join("Alfred.alfredpreferences");
    fs::create_dir_all(&alfred).unwrap();
    fs::create_dir_all(home.join("Library/Application Support/Alfred")).unwrap();
    fs::write(
        home.join("Library/Application Support/Alfred/prefs.json"),
        format!(r#"{{"current":"{}"}}"#, alfred.display()),
    )
    .unwrap();

    let output = run(
        home,
        &config,
        &["install", "--workflow-name", "test-workflow", "--retain"],
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let workflow = alfred.join("workflows/test-workflow");
    assert!(workflow.join("info.plist").is_file());
    assert_eq!(
        fs::read(workflow.join("icon.png")).unwrap(),
        include_bytes!("../assets/icon.png")
    );
    assert_eq!(
        fs::read_to_string(workflow.join("LICENSE")).unwrap(),
        include_str!("../../LICENSE")
    );
    let value = plist::Value::from_file(workflow.join("info.plist")).unwrap();
    let metadata = value.as_dictionary().unwrap();
    assert_eq!(
        metadata["version"].as_string(),
        Some(env!("CARGO_PKG_VERSION"))
    );
    assert_eq!(
        metadata["webaddress"].as_string(),
        Some(env!("CARGO_PKG_REPOSITORY"))
    );
    let objects = value.as_dictionary().unwrap()["objects"]
        .as_array()
        .unwrap();
    assert!(objects.len() > 2);
    assert!(objects.iter().any(|object| {
        object.as_dictionary().unwrap()["config"]
            .as_dictionary()
            .unwrap()
            .get("keyword")
            .and_then(|value| value.as_string())
            == Some("idea")
    }));
    assert!(objects.iter().any(|object| {
        object.as_dictionary().unwrap()["type"].as_string()
            == Some("alfred.workflow.trigger.action")
    }));
}

#[test]
fn export_works_without_alfred_and_includes_a_runnable_binary() {
    let temp = TempDir::new().unwrap();
    let home = temp.path();
    let config = home.join("config.toml");
    let workflow = home.join("exported workflow");

    let output = run(home, &config, &["export", workflow.to_str().unwrap()]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!home.join("Library/Application Support/Alfred").exists());
    let metadata = plist::Value::from_file(workflow.join("info.plist")).unwrap();
    assert_eq!(
        metadata.as_dictionary().unwrap()["bundleid"].as_string(),
        Some("com.mrchang0.alfred-jetbrains-rust")
    );
    assert_eq!(
        fs::read(workflow.join("icon.png")).unwrap(),
        include_bytes!("../assets/icon.png")
    );
    assert_eq!(
        fs::read_to_string(workflow.join("LICENSE")).unwrap(),
        include_str!("../../LICENSE")
    );
    let version = Command::new(workflow.join("bin/jb"))
        .arg("--version")
        .output()
        .unwrap();
    assert!(version.status.success());
    assert_eq!(
        String::from_utf8(version.stdout).unwrap().trim(),
        format!("jb {}", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn install_updates_an_imported_workflow_without_creating_a_duplicate() {
    let temp = TempDir::new().unwrap();
    let home = temp.path();
    let config = home.join("config.toml");
    let prefs = home.join("Alfred.alfredpreferences");
    let imported = prefs.join("workflows/user.workflow.imported");
    let output = run(home, &config, &["export", imported.to_str().unwrap()]);
    assert!(output.status.success());
    fs::write(imported.join("prefs.plist"), b"existing user preferences").unwrap();
    let info = imported.join("info.plist");
    let mut value = plist::Value::from_file(&info).unwrap();
    let metadata = value.as_dictionary_mut().unwrap();
    metadata.insert("version".into(), plist::Value::from("0.0.1"));
    metadata.insert("name".into(), plist::Value::from("My JetBrains"));
    let mut variables = plist::Dictionary::new();
    variables.insert("jb_terminal".into(), plist::Value::from("iTerm"));
    metadata.insert("variables".into(), plist::Value::Dictionary(variables));
    let search = metadata["objects"].as_array_mut().unwrap()[0]
        .as_dictionary_mut()
        .unwrap();
    let search_uid = search["uid"].as_string().unwrap().to_owned();
    let search_config = search
        .get_mut("config")
        .unwrap()
        .as_dictionary_mut()
        .unwrap();
    search_config.insert("keyword".into(), plist::Value::from("myjb"));
    search_config.insert("script".into(), plist::Value::from("outdated script"));
    let connection = metadata
        .get_mut("connections")
        .unwrap()
        .as_dictionary_mut()
        .unwrap()
        .get_mut(&search_uid)
        .unwrap()
        .as_array_mut()
        .unwrap()[0]
        .as_dictionary_mut()
        .unwrap();
    connection.insert("modifiers".into(), plist::Value::from(1048576));
    plist::to_file_xml(&info, &value).unwrap();
    let state = home.join(".config/alfred-jetbrains");
    fs::create_dir_all(&state).unwrap();
    fs::write(&config, "terminal = \"iTerm\"\n").unwrap();
    fs::write(state.join("pins.json"), br#"["/pinned/project"]"#).unwrap();
    fs::write(state.join("forgotten.json"), br#"["/hidden/project"]"#).unwrap();

    let alfred = home.join("Library/Application Support/Alfred");
    fs::create_dir_all(&alfred).unwrap();
    fs::write(
        alfred.join("prefs.json"),
        serde_json::json!({ "current": prefs }).to_string(),
    )
    .unwrap();

    let output = run(home, &config, &["install"]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!prefs.join("workflows/alfred-jetbrains-rust").exists());
    assert_eq!(
        fs::read(imported.join("prefs.plist")).unwrap(),
        b"existing user preferences"
    );
    let value = plist::Value::from_file(&info).unwrap();
    let metadata = value.as_dictionary().unwrap();
    assert_eq!(
        metadata["version"].as_string(),
        Some(env!("CARGO_PKG_VERSION"))
    );
    assert_eq!(metadata["name"].as_string(), Some("My JetBrains"));
    assert_eq!(
        metadata["variables"].as_dictionary().unwrap()["jb_terminal"].as_string(),
        Some("iTerm")
    );
    let search = metadata["objects"].as_array().unwrap()[0]
        .as_dictionary()
        .unwrap()["config"]
        .as_dictionary()
        .unwrap();
    assert_eq!(search["keyword"].as_string(), Some("myjb"));
    assert_eq!(
        search["script"].as_string(),
        Some("./bin/jb search --filter \"$1\"")
    );
    assert_eq!(
        metadata["connections"].as_dictionary().unwrap()[&search_uid]
            .as_array()
            .unwrap()[0]
            .as_dictionary()
            .unwrap()["modifiers"]
            .as_signed_integer(),
        Some(1048576)
    );
    assert_eq!(
        fs::read_to_string(&config).unwrap(),
        "terminal = \"iTerm\"\n"
    );
    for (file, contents) in [
        ("pins.json", r#"["/pinned/project"]"#),
        ("forgotten.json", r#"["/hidden/project"]"#),
    ] {
        assert_eq!(fs::read_to_string(state.join(file)).unwrap(), contents);
    }
    let output = run(home, &config, &["doctor"]);
    let diagnostics = String::from_utf8(output.stdout).unwrap();
    assert!(output.status.success());
    assert!(diagnostics.contains("user.workflow.imported (installed)"));

    let output = run(
        home,
        &config,
        &["install", "--workflow-name", "explicit-copy"],
    );
    assert!(output.status.success());
    assert!(prefs.join("workflows/explicit-copy/info.plist").is_file());
}

#[test]
fn install_does_not_overwrite_an_unreadable_workflow_definition() {
    let temp = TempDir::new().unwrap();
    let home = temp.path();
    let prefs = home.join("Alfred.alfredpreferences");
    let workflow = prefs.join("workflows/alfred-jetbrains-rust");
    fs::create_dir_all(workflow.join("bin")).unwrap();
    fs::write(workflow.join("info.plist"), "invalid plist").unwrap();
    fs::write(workflow.join("bin/jb"), "old CLI").unwrap();
    let alfred = home.join("Library/Application Support/Alfred");
    fs::create_dir_all(&alfred).unwrap();
    fs::write(
        alfred.join("prefs.json"),
        serde_json::json!({"current": prefs}).to_string(),
    )
    .unwrap();
    let output = run(home, &home.join("config.toml"), &["install"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("read existing workflow"));
    assert_eq!(
        fs::read_to_string(workflow.join("info.plist")).unwrap(),
        "invalid plist"
    );
    assert_eq!(
        fs::read_to_string(workflow.join("bin/jb")).unwrap(),
        "old CLI"
    );
}

#[test]
fn update_downloads_verified_releases_and_preserves_existing_installations_on_failure() {
    use std::{
        ffi::OsString,
        os::unix::fs::{MetadataExt, PermissionsExt, symlink},
    };

    for (version, failure) in [
        ("latest", "none"),
        ("v0.2.0", "none"),
        ("latest", "checksum"),
        ("latest", "download"),
        ("v0.2.0/../../other", "invalid"),
    ] {
        let temp = TempDir::new().unwrap();
        let home = temp.path();
        let config = home.join("config.toml");
        let prefs = home.join("Alfred preferences");
        let name = OsString::from("imported 'workflow' $(touch injected) 工作流");
        let workflow = prefs.join("workflows").join(&name);
        let output = Command::new(env!("CARGO_BIN_EXE_jb"))
            .arg("export")
            .arg(&workflow)
            .env("HOME", home)
            .env("JB_CONFIG", &config)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        // A second copy must not take precedence over the calling workflow.
        let other = prefs.join("workflows/alfred-jetbrains-rust");
        fs::create_dir_all(&other).unwrap();
        fs::write(other.join("info.plist"), "leave this workflow alone").unwrap();
        let alfred = home.join("Library/Application Support/Alfred");
        fs::create_dir_all(&alfred).unwrap();
        fs::write(
            alfred.join("prefs.json"),
            serde_json::json!({"current": prefs}).to_string(),
        )
        .unwrap();
        fs::write(workflow.join("prefs.plist"), "user preferences").unwrap();
        let original_info = fs::read(workflow.join("info.plist")).unwrap();

        let sources = home.join("release fixtures");
        fs::create_dir_all(&sources).unwrap();
        fs::copy(env!("CARGO_BIN_EXE_jb"), sources.join("jb")).unwrap();
        let archive = "jb-ARM64-apple-darwin.tar.gz";
        assert!(
            Command::new("tar")
                .args(["-czf"])
                .arg(sources.join(archive))
                .arg("-C")
                .arg(&sources)
                .arg("jb")
                .status()
                .unwrap()
                .success()
        );
        let sum = Command::new("shasum")
            .args(["-a", "256"])
            .arg(sources.join(archive))
            .output()
            .unwrap();
        assert!(sum.status.success());
        let sum = String::from_utf8(sum.stdout).unwrap();
        let checksum = if failure == "checksum" {
            "0".repeat(64)
        } else {
            sum.split_whitespace().next().unwrap().to_owned()
        };
        fs::write(
            sources.join("SHA256SUMS"),
            format!("{checksum}  {archive}\n"),
        )
        .unwrap();
        let mocks = home.join("mock commands");
        fs::create_dir_all(&mocks).unwrap();
        for (name, script) in [
            (
                "uname",
                "#!/bin/sh\ncase \"$1\" in -s) echo Darwin ;; -m) echo arm64 ;; esac\n",
            ),
            (
                "curl",
                r##"#!/bin/sh
while [ "$#" -gt 0 ]; do
  case "$1" in
    https://*) printf '%s\n' "$1" >> "$TEST_CURL_LOG"; file="${1##*/}" ;;
    -o) shift; destination="$1" ;;
  esac
  shift
done
[ "$TEST_DOWNLOAD_FAIL" != download ] || exit 22
/bin/cp "$TEST_SOURCES/$file" "$destination"
"##,
            ),
        ] {
            let path = mocks.join(name);
            fs::write(&path, script).unwrap();
            fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        }
        let downloads = home.join("downloads");
        let binary = if version == "v0.2.0" {
            let standalone = home.join("standalone CLI");
            fs::copy(env!("CARGO_BIN_EXE_jb"), &standalone).unwrap();
            standalone
        } else {
            workflow.join("bin/jb")
        };
        let inode = fs::metadata(&binary).unwrap().ino();
        let alias = home.join("terminal jb");
        symlink(&binary, &alias).unwrap();
        let mut command = Command::new(&alias);
        command.args(["update", version]);
        if version == "v0.2.0" {
            command.arg("--workflow-name").arg(&name);
        }
        let output = command
            .env("HOME", home)
            .env("JB_CONFIG", &config)
            .env(
                "PATH",
                format!("{}:{}", mocks.display(), std::env::var("PATH").unwrap()),
            )
            .env("TMPDIR", home)
            .env("TEST_CURL_LOG", &downloads)
            .env("TEST_SOURCES", &sources)
            .env("TEST_DOWNLOAD_FAIL", failure)
            .output()
            .unwrap();
        assert_eq!(
            output.status.success(),
            failure == "none",
            "{version}/{failure}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        if failure == "none" {
            assert_ne!(fs::metadata(&binary).unwrap().ino(), inode);
            let metadata = plist::Value::from_file(workflow.join("info.plist")).unwrap();
            assert_eq!(
                metadata.as_dictionary().unwrap()["version"].as_string(),
                Some(env!("CARGO_PKG_VERSION"))
            );
            let base = if version == "latest" {
                "latest/download"
            } else {
                "download/v0.2.0"
            };
            assert!(fs::read_to_string(&downloads).unwrap().contains(&format!(
                "{}/releases/{base}/{archive}",
                env!("CARGO_PKG_REPOSITORY")
            )));
        } else {
            assert_eq!(fs::metadata(&binary).unwrap().ino(), inode);
            assert_eq!(
                fs::read(workflow.join("info.plist")).unwrap(),
                original_info
            );
        }
        if failure == "invalid" {
            assert!(!downloads.exists());
        }
        assert_eq!(
            fs::read(&binary).unwrap(),
            fs::read(env!("CARGO_BIN_EXE_jb")).unwrap()
        );
        assert!(fs::symlink_metadata(&alias).unwrap().is_symlink());
        assert_eq!(
            fs::read_to_string(workflow.join("prefs.plist")).unwrap(),
            "user preferences"
        );
        assert_eq!(
            fs::read_to_string(other.join("info.plist")).unwrap(),
            "leave this workflow alone"
        );
        assert!(!fs::read_dir(home).unwrap().any(|entry| {
            entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with("alfred-jetbrains-rust.")
        }));
        assert_eq!(fs::read_dir(workflow.join("bin")).unwrap().count(), 1);
    }
}

#[test]
fn update_subcommand_takes_precedence_over_a_directory_named_update() {
    let temp = TempDir::new().unwrap();
    let home = temp.path();
    fs::create_dir(home.join("update")).unwrap();
    let output = run_in(home, home, &home.join("config.toml"), &["update", "--help"]);
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("jb update"));
}

#[test]
fn open_spawns_the_configured_binary() {
    let temp = TempDir::new().unwrap();
    let home = temp.path();
    write_fixture(home);
    let config = home.join("config.toml");
    let project = home.join("projects/alpha");
    let marker = home.join("opened.txt");
    let fake = home.join("fake-idea");
    fs::write(
        &fake,
        format!("#!/bin/sh\nprintf '%s' \"$1\" > '{}'\n", marker.display()),
    )
    .unwrap();
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&fake, fs::Permissions::from_mode(0o755)).unwrap();
    }

    let output = run(
        home,
        &config,
        &[
            "open",
            project.to_str().unwrap(),
            "--bin",
            fake.to_str().unwrap(),
        ],
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    for _ in 0..100 {
        if marker.exists() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    assert_eq!(
        fs::read_to_string(&marker).unwrap(),
        fs::canonicalize(&project).unwrap().to_string_lossy()
    );
}

#[test]
fn shorthand_opens_current_directory_with_inference() {
    let temp = TempDir::new().unwrap();
    let home = temp.path();
    write_fixture(home);
    let config = home.join("config.toml");
    let project = home.join("fresh-clone");
    fs::create_dir_all(&project).unwrap();
    fs::write(project.join("pom.xml"), "").unwrap();

    let marker = home.join("opened.txt");
    let idea = home.join("Applications/IntelliJ IDEA.app/Contents/MacOS/idea");
    fs::write(
        &idea,
        format!("#!/bin/sh\nprintf '%s' \"$1\" > '{}'\n", marker.display()),
    )
    .unwrap();
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&idea, fs::Permissions::from_mode(0o755)).unwrap();
    }

    let output = run_in(&project, home, &config, &["."]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    for _ in 0..100 {
        if marker.exists() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    assert_eq!(
        fs::read_to_string(&marker).unwrap(),
        fs::canonicalize(&project).unwrap().to_string_lossy()
    );
}

#[test]
fn global_flag_before_path_shorthand() {
    let temp = TempDir::new().unwrap();
    let home = temp.path();
    write_fixture(home);
    let config = home.join("config.toml");
    let project = home.join("fresh-clone");
    fs::create_dir_all(&project).unwrap();
    fs::write(project.join("pom.xml"), "").unwrap();

    let marker = home.join("opened.txt");
    let idea = home.join("Applications/IntelliJ IDEA.app/Contents/MacOS/idea");
    fs::write(
        &idea,
        format!("#!/bin/sh\nprintf '%s' \"$1\" > '{}'\n", marker.display()),
    )
    .unwrap();
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&idea, fs::Permissions::from_mode(0o755)).unwrap();
    }

    let output = run_in(&project, home, &config, &["--debug", "."]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    for _ in 0..100 {
        if marker.exists() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    assert_eq!(
        fs::read_to_string(&marker).unwrap(),
        fs::canonicalize(&project).unwrap().to_string_lossy()
    );
}

#[cfg(unix)]
#[test]
fn non_utf8_path_does_not_panic() {
    use std::{
        ffi::OsString,
        os::unix::ffi::{OsStrExt, OsStringExt},
    };

    let temp = TempDir::new().unwrap();
    let home = temp.path();
    write_fixture(home);
    let config = home.join("config.toml");
    let mut name = b"bad-".to_vec();
    name.push(0xff);
    let project = home.join("projects").join(OsString::from_vec(name));

    let marker = home.join("opened.txt");
    let idea = home.join("Applications/IntelliJ IDEA.app/Contents/MacOS/idea");
    fs::write(
        &idea,
        format!("#!/bin/sh\nprintf '%s' \"$1\" > '{}'\n", marker.display()),
    )
    .unwrap();
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&idea, fs::Permissions::from_mode(0o755)).unwrap();
    }

    let output = Command::new(env!("CARGO_BIN_EXE_jb"))
        .current_dir(home)
        .env("HOME", home)
        .env("JB_CONFIG", &config)
        .env("JB_BINARIES", home.join("empty-bin"))
        .arg(&project)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    for _ in 0..100 {
        if marker.exists() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    assert_eq!(
        fs::read(&marker).unwrap(),
        project.as_os_str().as_bytes(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn install_copies_the_binary_by_default() {
    let temp = TempDir::new().unwrap();
    let home = temp.path();
    write_fixture(home);
    let config = home.join("config.toml");

    let alfred = home.join("Alfred.alfredpreferences");
    fs::create_dir_all(&alfred).unwrap();
    fs::create_dir_all(home.join("Library/Application Support/Alfred")).unwrap();
    fs::write(
        home.join("Library/Application Support/Alfred/prefs.json"),
        format!(r#"{{"current":"{}"}}"#, alfred.display()),
    )
    .unwrap();

    let output = run(
        home,
        &config,
        &["install", "--workflow-name", "test-workflow"],
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let binary = alfred.join("workflows/test-workflow/bin/jb");
    assert!(binary.is_file());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_ne!(
            fs::metadata(&binary).unwrap().permissions().mode() & 0o111,
            0
        );
    }
}
