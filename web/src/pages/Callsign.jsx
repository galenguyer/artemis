import { useState } from 'react'
import { debounce } from 'lodash';
import './Callsign.css'
import Result from '../components/Result';

export default function Callsign() {
  const [results, setResults] = useState([]);

  const searchCallsign = debounce((e) => {
    console.log(e.target.value)
    fetch(`/api/v1/call/${e.target.value}`)
      .then(data => data.json())
      .then(data => {
        console.log(data);
        setResults(data);
      })
  }, 250);

  return (
    <div className="Callsign">
      <h2>Enter a callsign:</h2>
      <input id="callsign" onChange={searchCallsign}></input>
      <div id="results">
        {results.map(result => <Result result={result}></Result>
        )}
      </div>
    </div>
  )
}
