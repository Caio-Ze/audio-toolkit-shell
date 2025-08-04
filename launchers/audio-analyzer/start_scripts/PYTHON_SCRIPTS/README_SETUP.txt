BounceT4 One-click Setup (macOS)
================================

Purpose
-------
This single-step setup will:
1. Make every helper script executable.
2. Remove macOS quarantine flags from the included binaries.
3. Create (or activate) the **Python** virtual-environment that ships with the toolkit.
4. Install the packages listed in `requirements.txt` inside the venv.
5. Point the `ffmpeg` link to the correct build for **your** Mac (Apple-silicon or Intel).
6. Present a coloured success report.

After it finishes you will be able to double-click `start_scripts_rust` to open the tool menu.

Step-by-step (explain-it-like-I'm-five)
---------------------------------------
1. Find the folder called **PYTHON_SCRIPTS** that you just copied or un-zipped.
2. Right-click that folder and choose **"New Terminal at Folder"**.
   • If you do not see that option: open *Terminal.app*, type `cd ` (don't press <Enter> yet) and drag the **PYTHON_SCRIPTS** folder on top of the Terminal window – this pastes the path for you. Now press <Enter>.
3. Copy the blue command below, paste it in Terminal and press <Enter>:

```bash
sudo xattr -d com.apple.quarantine install_requirements_rust 2>/dev/null || true && \
chmod +x install_requirements_rust && ./install_requirements_rust
```
4. macOS will ask for **your computer login password** – type it and press <Enter>.  (You will not see the characters as you type. This is normal.)
5. Wait until the coloured **SETUP SUMMARY** appears.  Everything should show a green ✅.  If you see red ❌ messages, read them – they usually mean a missing Internet connection or permission that needs to be granted.
6. Close the Terminal window.

Using the toolkit
-----------------
• Double-click **start_scripts_rust** inside the *same* **PYTHON_SCRIPTS** folder.  A menu with all utilities will open.
• The first time you run each binary macOS might show "Unknown developer" – simply click **Open**.

Troubleshooting
---------------
✗ "start_scripts_rust gets killed immediately"  →  Run the setup again (step 3) to clear quarantine and ensure the correct `ffmpeg` link is in place.

✗ "command not found: python3"                 →  Ensure you are running macOS 10.15 Catalina or newer. Otherwise install Python 3 from python.org.

✗ Anything else                                →  Take a screenshot of the Terminal and send it to the BounceT4 team.

Happy bouncing! 🎉