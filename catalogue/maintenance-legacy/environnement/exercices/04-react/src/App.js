import React from "react";

const API = process.env.REACT_APP_API_URL;

export default function App() {
  return <h1>Planning ({API})</h1>;
}
