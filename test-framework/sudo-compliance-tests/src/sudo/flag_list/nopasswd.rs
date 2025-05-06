use sudo_test::{Command, Env, User, BIN_TRUE};

use crate::{GROUPNAME, USERNAME};

#[test]
fn when_invoking_user_is_root() {
    let argss: &[&[&str]] = &[
        &[],
        // regardless of `other_user`
        &["-U", USERNAME],
        // regardless of target user
        &["-u", USERNAME, "true"],
        // regardless of target group
        &["-g", "users", "true"],
    ];

    let env = Env("ALL ALL=(ALL:ALL) ALL").user(USERNAME).build();

    for args in argss {
        dbg!(args);
        let output = Command::new("sudo").arg("-l").args(*args).output(&env);
        let stdout = output.stdout();
        dbg!(&stdout);

        assert_not_contains!(stdout, "password");
        assert_not_contains!(stdout, "authenticate");
        if let ["-U", username] = args {
            assert_contains!(
                stdout,
                format!("User {username} may run the following commands on ")
            );
        } else if let [_, _, command] = args {
            assert_contains!(stdout, format!("/usr/bin/{command}"));
        } else {
            assert_contains!(stdout, "User root may run the following commands on ");
        }
    }
}

#[test]
fn when_target_user_is_self() {
    let other_user = "ghost";
    let env = Env("ALL ALL=(ALL:ALL) ALL")
        .user(USERNAME)
        .user(other_user)
        .build();

    let output = Command::new("sudo")
        .args(["-l", "-u", USERNAME, "true"])
        .as_user(USERNAME)
        .output(&env);
    let stdout = output.stdout();

    assert_not_contains!(stdout, "password");
    assert_not_contains!(stdout, "authenticate");
    assert_contains!(stdout, BIN_TRUE);
}

#[test]
fn when_invoking_user_belongs_to_target_group() {
    let other_user = "ghost";

    let env = Env("ALL ALL=(ALL:ALL) ALL")
        .user(User(USERNAME).secondary_group(GROUPNAME))
        .group(GROUPNAME)
        .user(other_user)
        .build();

    let output = Command::new("sudo")
        .args(["-l", "-g", GROUPNAME, "true"])
        .as_user(USERNAME)
        .output(&env);
    let stdout = output.stdout();

    assert_not_contains!(stdout, "password");
    assert_not_contains!(stdout, "authenticate");
    assert_contains!(stdout, BIN_TRUE);
}

#[test]
fn nopasswd_tag() {
    let hostname = "container";
    let env = Env(format!(
        "{USERNAME} {hostname}=(doesnt:matter) NOPASSWD: /does/not/matter
{USERNAME} {hostname}=(matters:not) /still/does/not/matter"
    ))
    .user(User(USERNAME).secondary_group(GROUPNAME))
    .group(GROUPNAME)
    .hostname(hostname)
    .build();

    let output = Command::new("sudo")
        .arg("-l")
        .as_user(USERNAME)
        .output(&env);
    let stdout = output.stdout();
    dbg!(&stdout);

    assert_not_contains!(stdout, "password");
    assert_not_contains!(stdout, "authenticate");
    assert_contains!(
        stdout,
        format!("User {USERNAME} may run the following commands on ")
    );
}
