import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/tauri";

const App = () => {
  const [shortcut, setShortcut] = useState("Ctrl+U");

  useEffect(() => {
    const unlisten = listen("hotkey-pressed", (event) => {
      const clipboardContent = event.payload as string;
      handleUpload(clipboardContent);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const handleShortcutChange: React.ChangeEventHandler<
    HTMLInputElement
  > = async (event) => {
    const newShortcut = event.target.value;
    setShortcut(newShortcut);
    await invoke("set_shortcut", { new_shortcut: newShortcut });
  };

  const handleUpload = async (content: string) => {
    const fileType = content.startsWith("data:image/") ? "image" : "text";
    await invoke("upload_file", { file_data: content, file_type: fileType });
    console.log(
      `${fileType.charAt(0).toUpperCase() + fileType.slice(1)} uploaded`
    );
  };

  return (
    <div className="App">
      <h1>System-wide Hotkey File Upload</h1>
      <label>
        Set Hotkey:
        <input type="text" value={shortcut} onChange={handleShortcutChange} />
      </label>
      <p>Press {shortcut} to upload the clipboard content</p>
    </div>
  );
};

export default App;
