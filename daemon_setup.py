import os
import subprocess

display = os.environ["DISPLAY"]
username = os.getlogin()
service_path = "/etc/systemd/system/screen_timed.service"

working_directory = os.getcwd() + "/daemon"
path_to_exec = working_directory + "/target/release/screen_timed"
service_file_contents = f"""[Unit]
Description=Screen Time Daemon
[Service]
ExecStart={path_to_exec}
WorkingDirectory={working_directory}
Restart=always
User={username}
Environment=DISPLAY={display}

[Install]
WantedBy=multi-user.target"""

print(f"This will be written to {service_path}: \n")
print(service_file_contents + "\n")
is_correct_file = input(
    "Is this correct (in particular the username, working directory, exec path)? (y/n):"
)
is_correct_file = is_correct_file.lower().strip()

if is_correct_file == "y":
    subprocess.call((f"systemctl unmask screen_timed.service"), shell=True)
    # create service file
    svc_file = open("/etc/systemd/system/screen_timed.service", "w")
    svc_file.write(service_file_contents)
    svc_file.close()

    print("Service file written. \n")
    print("Enabling daemon...")
    subprocess.call(
        (
            f"systemctl enable screen_timed.service && "
            f"systemctl daemon-reload && "
            f"systemctl restart screen_timed.service"
        ),
        shell=True,
    )
    print("Daemon should be enabled, take a look at the following logs... \n")
    subprocess.call(
        (f"systemctl status screen_timed.service"),
        shell=True,
    )
    print(
        "If the status is not active, run this command:`sudo journalctl -u screen-timed.service | less`. The logs should give you some idea of why this isn't working on your machine. \n"
    )
else:
    print("Exiting...")
