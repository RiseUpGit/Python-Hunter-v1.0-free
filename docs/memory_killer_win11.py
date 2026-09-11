import psutil
import logging
import subprocess
import sys
import time
import threading
import csv
from pathlib import Path
from datetime import datetime

# === Logging Setup ===
LOG_FILE = Path("memory_killer_win11.log")
CSV_FILE = Path("killed_processes.csv")
logging.basicConfig(
    filename=LOG_FILE,
    level=logging.INFO,
    format="%(asctime)s | %(message)s",
    datefmt="%Y-%m-%d %H:%M:%S"
)

# === Blacklist (Windows 11 safe defaults) ===
BLACKLIST = {
    "explorer.exe", "csrss.exe", "winlogon.exe", "lsass.exe", "smss.exe",
    "wininit.exe", "services.exe", "svchost.exe", "system", "dwm.exe",
    "taskmgr.exe", "cmd.exe", "powershell.exe", "python.exe", "pythonw.exe"
}
WHITELIST = set()

# === Detector Control ===
detector_running = False
stop_detector = threading.Event()

def log(msg):
    print(msg)
    logging.info(msg)

def is_blacklisted(name):
    return name.lower() in BLACKLIST

def is_whitelisted(name):
    return len(WHITELIST) == 0 or name.lower() in WHITELIST

def write_to_csv(pid, name, mem_mb, method, cmdline, reason="User confirmed"):
    """Append kill record to CSV"""
    file_exists = CSV_FILE.exists()
    with open(CSV_FILE, "a", newline="", encoding="utf-8") as f:
        writer = csv.writer(f)
        if not file_exists:
            writer.writerow(["Timestamp", "PID", "Name", "Memory_MB", "Method", "Command_Line", "Reason"])
        writer.writerow([
            datetime.now().strftime("%Y-%m-%d %H:%M:%S"),
            pid, name, round(mem_mb, 1), method,
            cmdline[:200] if cmdline else "",
            reason
        ])

def get_processes():
    procs = []
    for proc in psutil.process_iter(['pid', 'name', 'memory_info', 'cmdline']):
        try:
            info = proc.info
            mem_mb = info['memory_info'].rss / (1024 * 1024)
            cmdline = " ".join(info.get('cmdline') or []) if info.get('cmdline') else ""
            procs.append({
                'pid': info['pid'],
                'name': info['name'],
                'memory_mb': round(mem_mb, 1),
                'cmdline': cmdline
            })
        except:
            continue
    return sorted(procs, key=lambda x: x['memory_mb'], reverse=True)

def kill_process(pid, force=False):
    try:
        p = psutil.Process(pid)
        name = p.name()
        mem = p.memory_info().rss / (1024 * 1024)
        cmdline = " ".join(p.cmdline() or []) if p.cmdline() else ""

        if is_blacklisted(name):
            print(f"SKIPPED: {name} is blacklisted.")
            return False
        if not is_whitelisted(name):
            print(f"SKIPPED: {name} not in whitelist.")
            return False

        if force:
            subprocess.run(["taskkill", "/F", "/PID", str(pid)],
                capture_output=True, creationflags=subprocess.CREATE_NO_WINDOW)
            method = "taskkill /F"
        else:
            p.terminate()
            try:
                p.wait(timeout=4)
            except psutil.TimeoutExpired:
                subprocess.run(["taskkill", "/F", "/PID", str(pid)],
                    capture_output=True, creationflags=subprocess.CREATE_NO_WINDOW)
                method = "terminate → taskkill /F"

        log(f"KILLED | PID: {pid} | {name} | {mem:.1f} MB | {method}")
        write_to_csv(pid, name, mem, method, cmdline)
        return True
    except Exception as e:
        print(f"Error: {e}")
        return False

# === Background Python Spawn Detector ===
def python_spawn_detector():
    global detector_running
    known_pids = {p.pid for p in psutil.process_iter() if "python" in p.name().lower()}
    log("Python Spawn Detector started")

    while not stop_detector.is_set():
        try:
            current = {p.pid for p in psutil.process_iter() if "python" in p.name().lower()}
            new_pids = current - known_pids

            for pid in new_pids:
                try:
                    p = psutil.Process(pid)
                    mem = p.memory_info().rss / (1024 * 1024)
                    cmd = " ".join(p.cmdline() or [])
                    log(f"NEW PYTHON PROCESS | PID: {pid} | Mem: {mem:.1f} MB | Cmd: {cmd[:120]}")
                    print(f"\n[NEW PYTHON] PID {pid} ({mem:.1f} MB) — Kill? (y/n): ")
                    choice = input().lower()
                    if choice == "y":
                        kill_process(pid, force=True)
                except:
                    pass
            known_pids = current
            time.sleep(2)
        except:
            time.sleep(2)

    log("Python Spawn Detector stopped")
    detector_running = False

def start_detector():
    global detector_running, stop_detector
    if detector_running:
        print("Detector already running.")
        return
    stop_detector.clear()
    threading.Thread(target=python_spawn_detector, daemon=True).start()
    detector_running = True
    print("Python Spawn Detector started in background.")

def stop_detector_func():
    global detector_running
    if not detector_running:
        print("Detector is not running.")
        return
    stop_detector.set()
    detector_running = False
    print("Stopping detector...")

# === Windows 11 Memory Compression Menu ===
def windows_memory_compression_menu():
    if sys.platform != "win32":
        print("Windows-only feature.")
        return

    while True:
        print("\n=== Windows 11 Memory Compression Tools ===")
        print("1. Check Memory Compression status")
        print("2. Disable Memory Compression")
        print("3. Enable Memory Compression")
        print("4. Show detailed memory stats")
        print("5. Kill high-memory python processes only")
        print("6. Back")

        choice = input("\nSelect: ").strip()

        if choice == "1":
            subprocess.run(["powershell", "-Command", "Get-MMAgent | Select-Object MemoryCompression"])
        elif choice == "2":
            if input("Disable Memory Compression? (y/n): ").lower() == "y":
                subprocess.run(["powershell", "-Command", "Disable-MMAgent -MemoryCompression"])
                log("Memory Compression DISABLED")
        elif choice == "3":
            subprocess.run(["powershell", "-Command", "Enable-MMAgent -MemoryCompression"])
            log("Memory Compression ENABLED")
        elif choice == "4":
            subprocess.run(["powershell", "-Command", "Get-Counter '\\Memory\\*Compressed*'"])
        elif choice == "5":
            procs = [p for p in get_processes() if "python" in p['name'].lower()]
            for p in procs:
                print(f"{p['pid']} - {p['name']} ({p['memory_mb']} MB)")
            if input("Kill these Python processes? (y/n): ").lower() == "y":
                for p in procs:
                    kill_process(p['pid'], force=True)
        elif choice == "6":
            break

def main():
    print("=== Windows 11 Pro Memory Killer v5 (with CSV Export) ===")
    print("Killed processes are now logged to killed_processes.csv\n")

    while True:
        status = "RUNNING" if detector_running else "Stopped"
        print(f"\nMain Menu (Detector: {status})")
        print("1. Show top memory consumers")
        print("2. Kill by PID")
        print("3. Kill processes above X MB")
        print("4. Start Python Spawn Detector")
        print("5. Stop Python Spawn Detector")
        print("6. Windows Memory Compression Tools")
        print("7. View recent log")
        print("8. Exit")

        choice = input("\nSelect option: ").strip()

        if choice == "1":
            for p in get_processes()[:20]:
                print(f"{p['pid']:<8} {p['name']:<25} {p['memory_mb']:>8.1f} MB")

        elif choice == "2":
            try:
                pid = int(input("PID: "))
                force = input("Force kill? (y/n): ").lower() == "y"
                kill_process(pid, force=force)
            except ValueError:
                print("Invalid PID")

        elif choice == "3":
            try:
                threshold = float(input("Kill processes > __ MB: "))
                procs = [p for p in get_processes() if p['memory_mb'] > threshold]
                for p in procs:
                    if not is_blacklisted(p['name']):
                        print(f"{p['pid']} - {p['name']} ({p['memory_mb']} MB)")
                if input("Kill them? (y/n): ").lower() == "y":
                    for p in procs:
                        kill_process(p['pid'], force=True)
            except ValueError:
                print("Invalid number")

        elif choice == "4":
            start_detector()
        elif choice == "5":
            stop_detector_func()
        elif choice == "6":
            windows_memory_compression_menu()
        elif choice == "7":
            if LOG_FILE.exists():
                with open(LOG_FILE) as f:
                    print("".join(f.readlines()[-20:]))
        elif choice == "8":
            if detector_running:
                stop_detector_func()
            print("Exiting...")
            break

if __name__ == "__main__":
    main()
