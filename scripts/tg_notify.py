import os
import sys
import requests
import glob
import time
import subprocess
import html

def get_git_commit_message():
    try:
        cmd = ["git", "log", "-1", "--pretty=%B"]
        result = subprocess.check_output(cmd, text=True).strip()
        return result
    except Exception as e:
        print(f"Warning: Failed to get git commit message: {e}")
        return "No commit message available."

def reopen_topic(bot_token, chat_id, topic_id):
    """Attempt to reopen a closed forum topic."""
    url = f"https://api.telegram.org/bot{bot_token}/reopenForumTopic"
    data = {
        "chat_id": chat_id,
        "message_thread_id": topic_id
    }
    print(f"⚠️ Topic {topic_id} is closed. Attempting to reopen...")
    try:
        response = requests.post(url, data=data, timeout=30)
        if response.status_code == 200:
            print(f"✅ Topic {topic_id} successfully reopened!")
            return True
        else:
            print(f"❌ Failed to reopen topic: {response.text}")
            return False
    except Exception as e:
        print(f"❌ Exception reopening topic: {e}")
        return False

def send_telegram_file():
    bot_token = os.environ.get("TELEGRAM_BOT_TOKEN")
    chat_id = os.environ.get("TELEGRAM_CHAT_ID")
    
    topic_id = sys.argv[1] if len(sys.argv) > 1 else None
    event_label = sys.argv[2] if len(sys.argv) > 2 else "New Yield (新产物)"

    if not bot_token or not chat_id:
        print("Error: TELEGRAM_BOT_TOKEN or TELEGRAM_CHAT_ID not set.")
        sys.exit(1)

    repo = os.environ.get("GITHUB_REPOSITORY", "")
    run_id = os.environ.get("GITHUB_RUN_ID", "")
    server_url = os.environ.get("GITHUB_SERVER_URL", "https://github.com")
    run_url = f"{server_url}/{repo}/actions/runs/{run_id}"

    files = glob.glob("output/*.zip")
    if not files:
        print("Error: No grain sacks (zip files) found in output/.")
        sys.exit(1)
        
    file_path = files[0]
    file_name = os.path.basename(file_path)
    file_size = os.path.getsize(file_path) / 1024 / 1024

    print(f"Selecting yield: {file_name} ({file_size:.2f} MB)")
    commit_msg = get_git_commit_message()
    safe_commit_msg = html.escape(commit_msg)

    url = f"https://api.telegram.org/bot{bot_token}/sendDocument"
    
    caption = (
        f"🌾 <b>Meta-Hybrid: {event_label}</b>\n\n"
        f"⚖️ <b>重量 (Weight):</b> {file_size:.2f} MB\n\n"
        f"📝 <b>新性状 (Commit):</b>\n"
        f"<pre>{safe_commit_msg}</pre>\n\n"
        f"🚜 <a href='{run_url}'>查看日志 (View Log)</a>"
    )

    data = {
        "chat_id": chat_id,
        "caption": caption,
        "parse_mode": "HTML"
    }

    if topic_id and topic_id.strip() != "" and topic_id != "0":
        data["message_thread_id"] = topic_id
        print(f"Targeting Topic ID: {topic_id}")

    print(f"Dispatching yield to Granary (Telegram)...")
    
    max_retries = 2
    for attempt in range(max_retries):
        try:
            with open(file_path, "rb") as f:
                if attempt > 0:
                    f.seek(0)
                    
                files_payload = {"document": f}
                response = requests.post(url, data=data, files=files_payload, timeout=120)

            if response.status_code == 200:
                print("✅ Yield stored successfully!")
                return
            
            if response.status_code == 400 and "TOPIC_CLOSED" in response.text:
                if attempt < max_retries - 1:
                    if reopen_topic(bot_token, chat_id, topic_id):
                        print("🔄 Retrying upload in 2 seconds...")
                        time.sleep(2)
                        continue
                    else:
                        print("❌ Could not reopen topic. Aborting.")
                        sys.exit(1)
                else:
                    print("❌ Retries exhausted.")

            print(f"❌ Storage failed: {response.status_code} - {response.text}")
            sys.exit(1)
            
        except Exception as e:
            print(f"❌ Transport error: {e}")
            if attempt < max_retries - 1:
                time.sleep(2)
                continue
            sys.exit(1)

if __name__ == "__main__":
    send_telegram_file()