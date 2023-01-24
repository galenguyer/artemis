import { useState } from 'react';
import './Results.css';

export default function Result({ result }) {
    const [hidden, setHidden] = useState(true);
    const expand = () => {
        setHidden(!hidden);
    };

    let operator_class = "Unknown";
    if (result.operator_class === "T") {
        operator_class = "Technician";
    } else if (result.operator_class === "G") {
        operator_class = "General";
    } else if (result.operator_class === "E") {
        operator_class = "Amateur Extra";
    } else if (result.operator_class === "A") {
        operator_class = "Advanced";
    } else if (result.operator_class === "N") {
        operator_class = "Novice";
    }

    if (result.call_count > 1) {
        var call_history = result.call_history.split(",").filter(c => c != result.call_sign).join(", ");
    }

    return (
        <div className='Result' onClick={expand}>
            {result.first_name} {result.last_name} ({result.call_sign})
            <div className="ResultMore" style={{ display: hidden ? 'none' : 'block' }}>
                Location: {result.city}, {result.state}
                <br />
                Class: {operator_class}
                <br />
                Granted: {result.grant_date}
                <br />
                {(result.call_count > 1) ? `${result.call_count - 1} Prior Callsign${(result.call_count > 2 ? "s" : "")}: ${call_history}` : ""}
            </div>
        </div>
    )
}
