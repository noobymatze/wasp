import {useEffect, useState} from 'react'
import reactLogo from './assets/react.svg'
import viteLogo from '/vite.svg'
import './App.css'
import { main } from '../../wasm/pkg';

function App() {
  const [input, setInput] = useState("")
    const [data, setData] = useState("");

    const run = () => {
      setData(JSON.stringify(main(input), null, 4))
    }

  return (
    <>
      <div>
        <a href="https://vitejs.dev" target="_blank">
          <img src={viteLogo} className="logo" alt="Vite logo" />
        </a>
        <a href="https://react.dev" target="_blank">
          <img src={reactLogo} className="logo react" alt="React logo" />
        </a>
      </div>
      <h1>Vite + React</h1>
      <div className="card">
          <textarea value={input} onInput={(e) => setInput(e.currentTarget.value)} />
        <button onClick={() => run()}>
            Run
        </button>
          <pre>
              {data}
          </pre>
        <p>
          Edit <code>src/App.tsx</code> and save to test HMR
        </p>
      </div>
      <p className="read-the-docs">
        Click on the Vite and React logos to learn more
      </p>
    </>
  )
}

export default App
