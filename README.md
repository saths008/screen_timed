# screen-timed

Linux daemon that tracks and records time spent on active applications.
Other features include:

- Sending notifications every x minutes.

Data is processed and viewable through the [screen-time-app](https://github.com/saths008/screen-time-app).

## Set up

1. [Install Rust and cargo](https://www.rust-lang.org/tools/install)
2. Create a `screen_time_data.csv` in the root.

### Setting up the linux daemon:

2. This has been tested on Wayland and x11, (find out by doing `echo $XDG_SESSION_TYPE`):
<details>
<summary><b>More About x11 and Wayland</b></summary>

- This program has worked on a PC using wayland and Ubuntu but I run this daily on a PC using x11 and Ubuntu.
  - If you have a Nvidia graphics driver, you should probably use x11 over wayland if you aren't already (greater compatability). I was using wayland leading to errors getting the active window.

To switch to x11 on Ubuntu:

    - Go to settings > Colours (Device Colour Profiles) > Select `Standard Space - sRGB`
    - Edit `/etc/gdm3/custom.conf` and uncomment `WaylandEnable=false`
    - `sudo reboot`

If anything goes wrong, just undo this line: - Edit `/etc/gdm3/custom.conf` and uncomment `WaylandEnable=false`
and you should be back on wayland.

</details>

3. Create a screen-timed.service in /etc/systemd/system/ with content:

```
[Unit]
Description=Screen Time Daemon
[Service]
ExecStart=<pwd-to-screen-timed>/target/release/screen_timed
WorkingDirectory=<pwd-to-screen-timed>
Restart=always
User=<username>
Environment=DISPLAY=:0

[Install]
WantedBy=multi-user.target
```

Environment=DISPLAY=:1, replace :1 with whatever the output of `echo $DISPLAY` gives you.

4. To test if this works, in the main() of main.rs, insert:

```rust
    // should open up xclock, most linux distros have it installed
     match Command::new("xclock").output() {
         Ok(output) => {
             println!("output: {:?}", output);
         }
         Err(err) => {
             println!("Error: {}", err);
         }
     }
```

5. Enable the daemon:
   `sudo systemctl enable screen-timed.service`

6. To reload and restart the daemon thread:
   `sudo systemctl daemon-reload`
   `sudo systemctl restart screen-timed.service`
7. To view logs:
   `sudo journalctl -u screen-timed.service | less`
   To view the status:
   `sudo systemctl status screen-timed.service`

## Running Tests

`cargo test` or `cargo test -- --nocapture` to see stdout.
