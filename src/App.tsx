import { Fragment } from "react";

import "./App.css";

function App() {
  return (
    <Fragment>
      <h1 className="text-2xl font-semibold my-2">
        NanoUpload <span className="text-gray font-normal">v0.0.0</span>
      </h1>
      <p className="text-gray-400">
        Configure the hotkey used for NanoUpload here.
      </p>

      <div className="mt-4">
        <label htmlFor="hotkey" className="block text-gray-600">
          Hotkey
        </label>
        <input
          type="text"
          id="hotkey"
          className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-300 focus:ring focus:ring-blue-200 focus:ring-opacity-50"
        />
      </div>
    </Fragment>
  );
}

export default App;
