import { useState } from 'react'
import { debounce } from 'lodash';
import './Callsign.css'
import Result from '../components/Result';

export default function Search() {
  const [results, setResults] = useState([]);
  const [abortController, setAbortController] = useState(null);

  const search = () => {
    let params = new URLSearchParams();
    if (document.getElementById("callsign").value) {
      params.append("call_sign", document.getElementById("callsign").value);
    }
    if (document.getElementById("firstname").value) {
      params.append("first_name", document.getElementById("firstname").value);
    }
    if (document.getElementById("lastname").value) {
      params.append("last_name", document.getElementById("lastname").value);
    }
    if (params.toString() === "") {
      return;
    }

    if (abortController) {
      abortController.abort();
    }
    let controller = new AbortController();
    setAbortController(controller);

    fetch("/api/v1/search?" + params, { signal: controller.signal })
      .then(data => data.json())
      .then(data => {
        setResults(data);
        setAbortController(null);
      }).catch(err => {
        if (err.name !== "AbortError") {
          console.log(err);
        }
      });
  };

  return (
    <div className="Callsign">
      <h1>FCC Search</h1>
      <label htmlFor="callsign">Callsign: </label><input id="callsign" onChange={search}></input>
      <br />
      <label htmlFor="firstname">First Name: </label><input id="firstname" onChange={search}></input>
      <br />
      <label htmlFor="lastname">Last Name: </label><input id="lastname" onChange={search}></input>
      <br />
      <div id="results">
        {results.map(result => <Result key={result.call_sign + result.frn} result={result}></Result>
        )}
      </div>
    </div>
  )
}
