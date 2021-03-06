use crate::cli::*;
use crate::hostsfile::{ManagedHostsFile, MatchType};
use colored::*;
use hosts_parser::HostsFileLine;

pub fn show(summary: bool) {
    let hosts_file = ManagedHostsFile::must_load();
    if summary {
        println!("{}", hosts_file);
    } else {
        println!(
            "{}",
            hosts_file
                .without_comments()
                .iter()
                .map(|l| format_line(l))
                .collect::<Vec<String>>()
                .join("\n")
        );
    }
}
fn format_line(l: &HostsFileLine) -> String {
    if l.has_host() && l.has_comment() {
        format!(
            "{} {} {} {}",
            l.ip().unwrap().as_str().blue(),
            l.hosts().first().unwrap().as_str().green(),
            (&l.hosts()[1..]).join(" ").as_str().yellow(),
            l.comment().unwrap().cyan()
        )
    } else if l.has_host() {
        format!(
            "{} {} {}",
            l.ip().unwrap().as_str().blue(),
            l.hosts().first().unwrap().as_str().green(),
            (&l.hosts()[1..]).join(" ").as_str().yellow(),
        )
    } else {
        format!("{}", l.comment().unwrap().as_str().cyan())
    }
}

pub fn check(host: &str, exact: bool) {
    let hosts_file = ManagedHostsFile::must_load();
    let found = hosts_file.get_matches(host, &MatchType::from_bool(exact));
    println!(
        "{}",
        found
            .iter()
            .map(|l| format_line(l))
            .collect::<Vec<String>>()
            .join("\n")
    );
}

// pub fn add(ip: &str, names: &str, comment: &str) {
pub fn add(args: &Cli, sub_cmd: &CmdAdd) {
    let CmdAdd {
        names,
        ip,
        comment,
        update,
    } = sub_cmd;
    let all_names = names.split(',').collect::<Vec<&str>>();
    let mut hosts_file = ManagedHostsFile::must_load();
    let matches = hosts_file.get_multi_match(&all_names, &MatchType::Exact);
    if !matches.is_empty() && !update {
        println!(
            "The requested host is already present: \n{}",
            matches.join("\n")
        );
        return;
    }
    let names = all_names.join(" ");
    let comment = comment.join(" ");
    let computed_comment = if comment.is_empty() {
        "Added by hostman"
    } else {
        &comment
    };
    let host_line = format!("{} {} # {}", ip, names, computed_comment);
    if !matches.is_empty() {
        println!(
            "Updating host in hosts file: \n {} \n => {} {} {}",
            matches.join("\n"),
            ip,
            names,
            comment
        );
        for host in all_names {
            println!("Removing host {}", host);
            hosts_file.remove_host(host);
        }
    }

    println!("Adding {} {} to /etc/hosts", ip, names);
    let line = HostsFileLine::from_string(&host_line);
    if let Ok(line) = line {
        println!("{}", format_line(&line));
        hosts_file.add_line(&host_line);
        maybe_save(args.dry_run, hosts_file);
    } else {
        println!("Error parsing line: {}", host_line);
    }
}

pub fn add_local(args: &Cli, sub_cmd: &CmdAddLocal) {
    add(
        args,
        &CmdAdd {
            ip: String::from("127.0.0.1"),
            names: String::from(sub_cmd.names.as_str()),
            comment: sub_cmd.comment.clone(),
            update: sub_cmd.update,
        },
    )
}

pub fn remove(args: &Cli, host: &str) {
    let mut hosts_file = ManagedHostsFile::must_load();
    if !hosts_file.has_host(host) {
        println!("{} not in hosts file.", host);
        return;
    }
    println!("Removing host {}", host);
    hosts_file.remove_host(host);
    maybe_save(args.dry_run, hosts_file);
}

pub fn disable(args: &Cli, host: &str) {
    let mut hosts_file = ManagedHostsFile::must_load();
    if !hosts_file.has_host(host) {
        if hosts_file.has_disabled_host(host) {
            println!("{} is already disabled in hosts file.", host);
        } else {
            println!("{} is not in hosts file.", host);
        }
        return;
    }
    println!("Disabling host {}", host);
    hosts_file.disable_host(host);
    maybe_save(args.dry_run, hosts_file);
}

pub fn enable(args: &Cli, host: &str) {
    let mut hosts_file = ManagedHostsFile::must_load();
    if !hosts_file.has_disabled_host(host) {
        if hosts_file.has_host(host) {
            println!("{} is already enabled in hosts file.", host);
        } else {
            println!("{} is not in hosts file.", host);
        }
        return;
    }
    println!("Enabling host {}", host);
    hosts_file.enable_host(host);
    maybe_save(args.dry_run, hosts_file);
}

pub fn update() {
    let target = self_update::get_target();
    let status = self_update::backends::github::Update::configure()
        .repo_owner("lucascaro")
        .repo_name("hostman")
        .target(&target)
        .bin_name("hostman")
        .show_download_progress(true)
        .current_version(cargo_crate_version!())
        .build()
        .expect("cannot build")
        .update()
        .expect("cannot update");
    println!("Update status: `{}`!", status.version());
}

fn maybe_save(dry_run: bool, hosts_file: ManagedHostsFile) {
    if dry_run {
        println!("{}", hosts_file);
    } else {
        hosts_file.save();
    }
}
