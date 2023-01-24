import { BrowserRouter, Route, Routes } from 'react-router-dom'
import Callsign from './pages/Callsign';
import Search from './pages/Search';
import './App.css'

export default function App() {
  return <>
    <BrowserRouter>
      <Routes>
        <Route path="/search" element={<Search />} />
        <Route path="/callsign" element={<Callsign />} />
        <Route path="/" exact element={<Search />} />
      </Routes>
    </BrowserRouter>
  </>
}
