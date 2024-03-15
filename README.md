# screen-timed

Ubuntu doesn't come with a screen time tracker and much of the software available is too complex or doesn't provide a nice UI. This project aims to provide a screen time tracker for Ubuntu by monitoring the active windows.

![Screenshot 1](/docs/screenshots/week-screenshot.png)
![Screenshot 2](/docs/screenshots/day-screenshot.png)

## Project Structure

There are 2 folders:

- daemon
- desktop-app

The daemon folder contains a Linux daemon that tracks and records time spent on active applications. The Linux daemon also comes with features such as sending screen time reminders every x minutes.

The desktop app folder contains the desktop app which process screen time data to make it viewiable through graphs, etc.

## Set up of daemon

1. [Install Rust and cargo](https://www.rust-lang.org/tools/install)

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

3.  To test if your env display works, in the main() of main.rs, insert into main.rs so:

```rust
fn main() {
    // screen_timed::run().unwrap();

     match Command::new("xclock").output() {
         Ok(output) => {
             println!("output: {:?}", output);
         }
         Err(err) => {
             println!("Error: {}", err);
         }
     }
}
```

4. Run `cargo run` and see if xclock opens. If it does, you're good to go. Go ahead and return main.rs to its original state.

5. Now in the root, run `sudo python3 daemon_setup.py`. This requires sudo as it creates a systemd service.

6. If you would like to remove the daemonm run `sudo python3 daemon_removal.py`.

7. To view logs:
   `sudo journalctl -u screen_timed.service | less`
   To view the status:
   `sudo systemctl status screen_timed.service`

8. To run the tests for the daemon:

- `cd daemon`
- `cargo test` or `cargo test -- --nocapture` to see stdout.

## Set up of desktop-app

1. Refer to the latest release on the GitHub releases page for the .deb file.
   To install a .deb file:

- `sudo dpkg -i /path/to/deb/file`

2. Building from source / Just running the desktop app:

- [Install Rust and cargo](https://www.rust-lang.org/tools/install)
- [Install Tauri](https://tauri.app/)
- [Install node](https://github.com/nvm-sh/nvm)
- `cargo install tauri-cli`
- To run:
  `cargo tauri dev`

- To bundle:
  `cargo tauri build`

- You can now find the appimage or the deb here: `<pwd-to-screen_timed>/desktop_app/src-tauri/target/release/bundle/`
