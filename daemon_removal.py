import os
import subprocess

subprocess.call(
    (
        f"systemctl stop screen_timed.service && systemctl disable screen_timed.service && systemctl unmask screen_timed.service"
    ),
    shell=True,
)
subprocess.call(
    (f"systemctl daemon-reload"),
    shell=True,
)

print("Daemon stopped, removing service file... \n")
os.remove("/etc/systemd/system/screen_timed.service")
print("Service file removed. \n")
