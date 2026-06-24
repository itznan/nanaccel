import os
import subprocess
import sys

def main():
    work_dir = r"E:\NAN\Github\nanaccel\nanaccel-windows-x86_64"
    exe_path = os.path.join(work_dir, "nanaccel.exe")
    input_file = os.path.join(work_dir, "classroom.mp4")
    output_transcoded = os.path.join(work_dir, "classroom_transcoded.mp4")
    output_screenshot = os.path.join(work_dir, "classroom_screenshot.png")
    
    if not os.path.exists(exe_path):
        print(f"Error: Executable not found at {exe_path}")
        sys.exit(1)
        
    print(f"Testing executable: {exe_path}")
    
    # 1. Verification of info command
    print("\n[1/3] Testing info command...")
    res = subprocess.run([exe_path, "info"], capture_output=True, text=True)
    print(res.stdout)
    if "NVIDIA GPU Status" not in res.stdout:
        print("FAILED: GPU info command did not return GPU status")
        sys.exit(1)
    print("SUCCESS: Info command verified!")

    # Clean previous outputs if they exist
    for path in [output_transcoded, output_screenshot]:
        if os.path.exists(path):
            os.remove(path)

    # 2. Verification of transcode command
    print("\n[2/3] Testing transcode command...")
    transcode_args = [exe_path, "transcode", input_file, output_transcoded, "-c", "h264"]
    res = subprocess.run(transcode_args, capture_output=True, text=True)
    if os.path.exists(output_transcoded) and os.path.getsize(output_transcoded) > 0:
        print(f"SUCCESS: Transcoded file created ({os.path.getsize(output_transcoded)} bytes)")
    else:
        print("FAILED: Transcode command did not produce valid output.")
        print("STDOUT:", res.stdout)
        print("STDERR:", res.stderr)
        sys.exit(1)

    # 3. Verification of screenshot command
    print("\n[3/3] Testing screenshot command...")
    screenshot_args = [exe_path, "screenshot", input_file, output_screenshot, "-t", "1000"]
    res = subprocess.run(screenshot_args, capture_output=True, text=True)
    if os.path.exists(output_screenshot) and os.path.getsize(output_screenshot) > 0:
        print(f"SUCCESS: Screenshot created ({os.path.getsize(output_screenshot)} bytes)")
    else:
        print("FAILED: Screenshot command did not produce valid output.")
        print("STDOUT:", res.stdout)
        print("STDERR:", res.stderr)
        sys.exit(1)

    print("\nALL TESTS PASSED SUCCESSFULLY!")

if __name__ == "__main__":
    main()
