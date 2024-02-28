# screen-timed

Linux daemon that tracks and records time spent on active applications.

Data is processed and viewable through the screen-time-app which is under a different repo.

## Set up

1. Make sure Rust's Cargo tools are installed.
2. Setting up the linux daemon:

If you are using x11 (find out by doing `echo $XDG_SESSION_TYPE`):

a. Create a screen-timed.service in /etc/systemd/system/ with content:

```
[Unit]
Description=Screen Time Daemon
[Service]
ExecStart=<pwd-to-screen-timed>/target/release/screen-timed
WorkingDirectory=<pwd-to-screen-timed>
Restart=always
User=<username>
Environment=DISPLAY=:1

[Install]
WantedBy=multi-user.target
```

Environment=DISPLAY=:1, replace :1 with whatever the output of `echo $DISPLAY` gives you.

To test if this works, in the main() of main.rs, insert:

```rust
    // should open up xclock which uses x11.
     match Command::new("xclock").output() {
         Ok(output) => {
             println!("output: {:?}", output);
         }
         Err(err) => {
             println!("Error: {}", err);
         }
     }

```

2. `sudo systemctl daemon-reload`
   `sudo systemctl restart screen-timed.service`
