use anyhow::Result;
use std::process::{Command, Child, Stdio};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tokio::time::{sleep, Duration};
use tracing::{info, warn, error};

#[allow(dead_code)]
pub struct ProcessManager {
    processes: Arc<Mutex<HashMap<String, Child>>>,
}

#[allow(dead_code)]
impl ProcessManager {
    pub fn new() -> Self {
        Self {
            processes: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Start a new process and track it
    pub fn start_process(&self, name: String, command: &str, args: &[&str]) -> Result<u32> {
        info!("Starting process: {} with command: {} {:?}", name, command, args);
        
        let mut cmd = Command::new(command);
        cmd.args(args);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        
        let child = cmd.spawn()?;
        let pid = child.id();
        
        {
            let mut processes = self.processes.lock().unwrap();
            processes.insert(name, child);
        }
        
        info!("Process started with PID: {}", pid);
        Ok(pid)
    }
    
    /// Stop a tracked process
    pub fn stop_process(&self, name: &str) -> Result<bool> {
        info!("Stopping process: {}", name);
        
        let mut processes = self.processes.lock().unwrap();
        if let Some(mut child) = processes.remove(name) {
            child.kill()?;
            match child.wait() {
                Ok(status) => {
                    info!("Process {} stopped with status: {:?}", name, status);
                    Ok(true)
                }
                Err(e) => {
                    warn!("Error waiting for process {} to stop: {}", name, e);
                    Ok(true) // Process is killed anyway
                }
            }
        } else {
            warn!("Process {} not found", name);
            Ok(false)
        }
    }
    
    /// Check if a process is running
    pub fn is_process_running(&self, name: &str) -> bool {
        let mut processes = self.processes.lock().unwrap();
        if let Some(child) = processes.get_mut(name) {
            match child.try_wait() {
                Ok(Some(_status)) => {
                    // Process has exited
                    processes.remove(name);
                    false
                }
                Ok(None) => {
                    // Process is still running
                    true
                }
                Err(e) => {
                    warn!("Error checking process status for {}: {}", name, e);
                    processes.remove(name);
                    false
                }
            }
        } else {
            false
        }
    }
    
    /// Get the PID of a tracked process
    pub fn get_process_pid(&self, name: &str) -> Option<u32> {
        let processes = self.processes.lock().unwrap();
        processes.get(name).map(|child| child.id())
    }
    
    /// Stop all tracked processes
    pub fn stop_all_processes(&self) -> Result<()> {
        info!("Stopping all processes");
        
        let mut processes = self.processes.lock().unwrap();
        let names: Vec<String> = processes.keys().cloned().collect();
        
        for name in names {
            if let Some(mut child) = processes.remove(&name) {
                match child.kill() {
                    Ok(_) => {
                        info!("Killed process: {}", name);
                        let _ = child.wait(); // Wait for cleanup
                    }
                    Err(e) => {
                        warn!("Error killing process {}: {}", name, e);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Check if a command exists on the system
    pub fn command_exists(command: &str) -> bool {
        match Command::new("which")
            .arg(command)
            .output()
        {
            Ok(output) => output.status.success(),
            Err(_) => {
                // Try with 'where' on Windows
                match Command::new("where")
                    .arg(command)
                    .output()
                {
                    Ok(output) => output.status.success(),
                    Err(_) => false,
                }
            }
        }
    }
    
    /// Execute a command and return output
    pub async fn execute_command(command: &str, args: &[&str]) -> Result<String> {
        info!("Executing command: {} {:?}", command, args);
        
        let output = tokio::process::Command::new(command)
            .args(args)
            .output()
            .await?;
        
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok(stdout.to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("Command failed: {}", stderr))
        }
    }
    
    /// Monitor a process and restart if it crashes
    pub async fn monitor_process(&self, name: String, command: String, args: Vec<String>, restart_delay: Duration) {
        let name_clone = name.clone();
        let command_clone = command.clone();
        let args_clone = args.clone();
        
        tokio::spawn(async move {
            loop {
                info!("Monitoring process: {}", name_clone);
                
                // Check if process is running
                let manager = ProcessManager::new(); // Would use shared instance in real implementation
                if !manager.is_process_running(&name_clone) {
                    warn!("Process {} is not running, attempting restart", name_clone);
                    
                    // Attempt to restart
                    let args_str: Vec<&str> = args_clone.iter().map(|s| s.as_str()).collect();
                    match manager.start_process(name_clone.clone(), &command_clone, &args_str) {
                        Ok(pid) => {
                            info!("Successfully restarted process {} with PID: {}", name_clone, pid);
                        }
                        Err(e) => {
                            error!("Failed to restart process {}: {}", name_clone, e);
                        }
                    }
                }
                
                sleep(restart_delay).await;
            }
        });
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ProcessManager {
    fn drop(&mut self) {
        if let Err(e) = self.stop_all_processes() {
            warn!("Error stopping processes on drop: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_command_exists() {
        // Test with a command that should exist on most systems
        assert!(ProcessManager::command_exists("echo"));
        
        // Test with a command that shouldn't exist
        assert!(!ProcessManager::command_exists("non_existent_command_12345"));
    }
    
    #[tokio::test]
    async fn test_execute_command() {
        let result = ProcessManager::execute_command("echo", &["hello"]).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "hello");
    }
}