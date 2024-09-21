import React from 'react';
import { ConfigProvider } from 'antd';
import { BrowserRouter as Router, Route, Routes } from 'react-router-dom';
import Home from './pages/Home';
import Contents from './pages/Content';
import ContentDetails from './pages/ContentDetails';
import ProviderDashboard from './pages/ProviderDashboard';
import 'antd/dist/reset.css';
import PeerDashboard from './pages/PeerDashboard';
import CallbackPage from "./pages/CallbackPage";
import LoginPage from "./pages/LoginPage";

// import { invoke } from "@tauri-apps/api/tauri";

const App: React.FC = () => {

    const theme = {
        token: {
            colorPrimary: '#AD49E1',   // Main color
            colorBgBase: '#EBD3F8',     // Background color
            colorTextBase: '#2E073F',   // Primary text color
            colorTextSecondary: '#7A1CAC', // Secondary text color
        },
    };
    return (
        <ConfigProvider theme={theme}>
            <div className="App">
                <Router>
                    <Routes>
                        <Route path="/" element={<Home />} />
                        <Route path="/login" element={<LoginPage />} />
                        <Route path="/callback" element={<CallbackPage />} />
                        <Route path="/contents" element={<Contents />} />
                        <Route path="/dashboard" element={<ProviderDashboard />} />
                        <Route path="/content/:id" element={<ContentDetails />} />
                        <Route path="/node-dashboard" element={<PeerDashboard />} />
                    </Routes>
                </Router>
            </div>
        </ConfigProvider>
    );
};

export default App;
