# 🤖 xcom-rs - Simple Agentic AI Twitter Tool

[![Download xcom-rs](https://img.shields.io/badge/Download-xcom--rs-blue?style=for-the-badge&logo=github)](https://github.com/kunalmchandak/xcom-rs/releases)

---

## 📘 What is xcom-rs?

xcom-rs is a command-line tool that lets you work with Twitter’s API in a smart, automated way. It’s made for easy machine use, which means it can handle tasks without asking you questions all the time. The commands are built to give consistent results and to safely do the same action multiple times if you need.

This tool is helpful if you want to manage Twitter data or automate tasks using a clear and predictable interface. You don’t need to open Twitter’s website or install complex software. Just run commands from your computer and get fast results.

---

## 💡 Key Features

- **Agentic AI-friendly:** Works well with AI tools and scripts for automation.
- **Command-line Interface (CLI):** Use simple typed commands instead of complex programs.
- **Machine-readable output:** Results come in a clear format that's easy to read or use in other programs.
- **Non-interactive operations:** Runs commands without needing you to respond or make choices each time.
- **Idempotent commands:** Repeating a command won’t cause unexpected changes.
- **Built with Rust:** Stable and efficient software.

---

## 🖥️ System Requirements

To use xcom-rs, your computer must meet these requirements:

- Operating System: Windows 10 or later, macOS 10.14 or later, or a popular 64-bit Linux distribution
- Free Disk Space: At least 50 MB
- RAM: Minimum 1 GB (2 GB recommended)
- Internet Connection: Required to access Twitter’s API
- Terminal or Command Prompt access (comes built-in with your operating system)

You don’t need to install any extra software or programming tools.

---

## 🚀 Getting Started

Follow these steps to download and run xcom-rs on your computer. No coding skills needed.

---

## 📥 Download & Install

1. Visit the official release page by clicking the big button below:

   [![Download xcom-rs](https://img.shields.io/badge/Download-xcom--rs-blue?style=for-the-badge&logo=github)](https://github.com/kunalmchandak/xcom-rs/releases)

2. On the page, find the latest release version. It will usually be at the top of the list.

3. Download the file that matches your operating system:
   - For Windows, look for a file ending in `.exe`.
   - For macOS, find a `.dmg` or `.tar.gz` file.
   - For Linux, download an `.AppImage` or `.tar.gz` file.

4. Save the file to a place you can easily find, like your Desktop or Downloads folder.

5. To install:
   - On Windows: Double-click the `.exe` file and follow the steps on the screen if any appear.
   - On macOS: Open the `.dmg` file and drag xcom-rs to your Applications folder.
   - On Linux: Extract the `.tar.gz` file and follow the instructions in the included README, or make the `.AppImage` executable by right-clicking, selecting Properties, then Permissions, and checking "Allow executing file as program".

6. No further software is needed. You are ready to run xcom-rs.

---

## ▶️ Running xcom-rs

Here is how to launch and use the application:

1. Open your computer’s terminal or command prompt:
   - Windows: Press `Win + R`, type `cmd`, then press Enter.
   - macOS: Open Finder, go to Applications > Utilities, then double-click Terminal.
   - Linux: Launch Terminal from your apps menu.

2. Type `xcom-rs` and press Enter. You should see a list of commands and help information.

3. To use a command, type it exactly as shown. For example, getting your Twitter user info might look like:

   ```
   xcom-rs user info --username your_twitter_name
   ```

4. Commands output clear, readable results that you can use or save to a file.

---

## 🤔 How to Use Commands

Commands in xcom-rs perform Twitter actions through simple instructions. Here are common uses:

- **Check account details:** See basic info about a username.
- **Post a tweet:** Send a new tweet from your account.
- **Retrieve tweets:** Get recent tweets from any user.
- **Follow/unfollow users:** Manage your Twitter connections without opening the website.

Each command runs by itself and does not ask for extra input, except for the parts you type in your command line.

---

## 🔧 Configuration and Authentication

To connect to Twitter, xcom-rs needs your permission token:

1. Go to Twitter’s developer website and create an app to get API keys.
2. Copy the keys and set them as environment variables on your computer:
   - On Windows, search for “Environment Variables” and add new variables `TWITTER_API_KEY`, `TWITTER_API_SECRET`, `TWITTER_ACCESS_TOKEN`, and `TWITTER_ACCESS_SECRET`.
   - On macOS/Linux, add these lines to your terminal profile file `~/.bash_profile` or `~/.zshrc`:
     ```
     export TWITTER_API_KEY="your_key"
     export TWITTER_API_SECRET="your_secret"
     export TWITTER_ACCESS_TOKEN="your_access_token"
     export TWITTER_ACCESS_SECRET="your_access_secret"
     ```
3. Restart your terminal or command prompt to load the new variables.

This setup is one-time and lets xcom-rs securely talk to Twitter on your behalf.

---

## 🛠️ Troubleshooting Tips

- If the program does not start, double-check you downloaded the correct version for your system.
- Ensure your environment variables are set correctly for authentication.
- If commands return errors, check your internet connection.
- Update to the latest release if you notice strange behavior.
- Visit the repository’s Issues page on GitHub to see if others have the same problem.

---

## 📄 License & Contributors

xcom-rs is open source and free to use under the MIT License. The project is maintained by its developer and community contributors.

For more details, visit the GitHub repository: [https://github.com/kunalmchandak/xcom-rs](https://github.com/kunalmchandak/xcom-rs)

---

## 🔗 Useful Links

- Official Releases: [https://github.com/kunalmchandak/xcom-rs/releases](https://github.com/kunalmchandak/xcom-rs/releases)
- Documentation and examples inside the GitHub repo.
- Twitter Developer Portal for API keys setup.

---

Thank you for choosing xcom-rs. This tool aims to give you smart and simple access to Twitter through your computer.