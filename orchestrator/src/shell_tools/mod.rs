use std::process::{Command, Stdio};

//TODO: simplify boolean expressions between not_found() and failure() everywhere

pub trait RunInfo {
    /// Returns false if the command was not found on the system, or in a general I/O error
    fn not_found(&self) -> bool;

    /// Returns true if the command returned a non-zero exit code
    fn non_zero_exit(&self) -> bool;

    /// Returns true if there was a failure of the command (either not found or non-zero exit code)
    fn failure(&self) -> bool;

    fn panic_on_not_found(&self);

    fn panic_on_nonzero_exit(&self);

    fn panic_on_failure(&self);
}

struct RunDetails {
    stdout: String,
    stderr: String,
    exit_code: isize,
}

pub struct RunResult {
    result: Option <RunDetails>,
    command: String,
    shell: String,
}

impl RunInfo for RunResult {
    /// Returns false if the command was not found on the system, or in a general I/O error
    fn not_found(&self) -> bool {
        self.result.is_none()
    }

    /// Returns true if the command returned a non-zero exit code
    fn non_zero_exit(&self) -> bool {
        self.result.is_some() && self.result.as_ref().unwrap().exit_code != 0
    }

    /// Returns true if there was a failure of the command (either not found or non-zero exit code)
    fn failure(&self) -> bool {
        self.not_found() | self.non_zero_exit()
    }

    fn panic_on_not_found(&self) {
        if self.not_found() {
            panic!("Error: could not run command {} with shell {}", self.command, self.shell);
        }
    }

    fn panic_on_nonzero_exit(&self) {
        if self.non_zero_exit() {
            panic!("Error: command returned non-zero code ({}): {}\nOutput:\n{}\n\n{}", self.result.as_ref().unwrap().exit_code, self.command, self.result.as_ref().unwrap().stdout, self.result.as_ref().unwrap().stderr);
        }
    }

    fn panic_on_failure(&self) {
        self.panic_on_nonzero_exit();
        self.panic_on_not_found();
    }
}

impl RunResult {
    #[allow(dead_code)]
    pub fn stdout(&self) -> &str {
        &self.result.as_ref().unwrap().stdout
    }

    #[allow(dead_code)]
    pub fn stderr(&self) -> &str {
        &self.result.as_ref().unwrap().stderr
    }

    #[allow(dead_code)]
    pub fn error_code(&self) -> isize {
        self.result.as_ref().unwrap().exit_code
    }

    #[allow(dead_code)]
    pub fn check_success(&self) -> bool {
        self.result.is_some() && self.result.as_ref().unwrap().exit_code == 0
    }

    #[allow(dead_code)]
    pub fn exists(&self) -> bool {
        self.result.is_some()
    }

    #[allow(dead_code)]
    pub fn has_output(&self) -> bool {
        self.result.is_some() && {
            let res_det = self.result.as_ref().unwrap();
            res_det.stderr == "" && res_det.stdout == ""
        }
    }
}

pub struct RunSummary {
    exit_code: isize,
    did_run: bool,
    command: String,
}

impl RunSummary {
    pub fn exit_code(&self) -> isize {
        self.exit_code
    }
}

impl RunInfo for RunSummary {

    /// Returns false if the command was not found on the system, or in a general I/O error
    fn not_found(&self) -> bool {
        !self.did_run
    }

    /// Returns true if the command returned a non-zero exit code
    fn non_zero_exit(&self) -> bool {
        self.did_run && self.exit_code !=0
    }

    /// Returns true if there was a failure of the command (either not found or non-zero exit code)
    fn failure(&self) -> bool {
        self.not_found() | self.non_zero_exit()
    }

    fn panic_on_not_found(&self) {
        if self.not_found() {
            panic!("Error: could not run command {}", self.command);
        }
    }

    fn panic_on_nonzero_exit(&self) {
        if self.non_zero_exit() {
            panic!("Error: command returned non-zero code ({}): {}", self.exit_code, self.command);
        }
    }

    fn panic_on_failure(&self) {
        self.panic_on_nonzero_exit();
        self.panic_on_not_found();
    }
}

pub fn check_command_exist(command: &str) -> bool {
    assert!(!command.contains(" "));

    match Command::new(command).output() {
        Ok(_) => true,
        Err(_) => false
    }
}

pub fn run_command(command: &str, shell: &str) -> RunResult {
    let output = Command::new(shell)
            .arg("-c")
            .arg(command)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output();

    let mut ret = RunResult {
        result: None,
        command: String::from(command),
        shell: String::from(shell)
    };
    match output {
        Ok(o) => ret.result = Some(RunDetails {
                stdout: String::from_utf8_lossy(&o.stdout).to_string(),
                stderr: String::from_utf8_lossy(&o.stderr).to_string(),
                exit_code: o.status.code().unwrap() as isize,
        }),
        Err(_) => (),
    }
    ret
}

/// Fast version of run_command() (does not record stdout and stderr)
pub fn run_command_no_output(command: &str, shell: &str) -> RunSummary {
    let output = Command::new(shell)
            .arg("-c")
            .arg(command)
            .output();

    let ret = RunSummary {
        did_run: output.is_ok(),
        exit_code: { if output.is_err() {0} else {output.unwrap().status.code().unwrap() as isize} },
        command: String::from(command),
    };
    ret
}

/// Will try to run a command until success n times, before giving up
pub fn run_command_try_again(command: &str, shell: &str, num_tries: usize, wait_time: Option<std::time::Duration>) -> RunSummary {
    //assert(num_tries > 0);
    let mut counter = num_tries;

    let mut test = RunSummary{
        exit_code: 0,
        did_run: false,
        command: "".to_string(),
    };

    while counter > 0 {
        test = run_command_no_output(command, shell);
        match test.failure() {
            true => println!("Warning: command {} failed to execute, trying again (try {} out of {})", command, num_tries-counter+1, num_tries),
            false => return test,
        }
        counter-=1;
        wait_time.map(|d| std::thread::sleep(d));
    }
    test
}

pub fn run_command_interactive(command: &str, shell: &str) -> RunSummary {
    let output = Command::new(shell).arg("-c")
            .arg(command)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output();

    let ret = RunSummary {
        did_run: output.is_ok(),
        exit_code: { if output.is_err() {0} else {output.unwrap().status.code().unwrap() as isize} },
        command: String::from(command),
    };
    ret
}

// Will hold all variables created with set_global_variables
// static vars: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));

// pub fn set_global_variable(name: &str, value: &str) {
// }