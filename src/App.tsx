import React from 'react';
import { ConfigProvider } from 'antd';
import { BrowserRouter as Router, Route, Routes } from 'react-router-dom';
import Home from './pages/Home';
import Contents from './pages/Content';
import ContentDetails from './pages/ContentDetails';
import ProviderDashboard from './pages/ProviderDashboard';
import 'antd/dist/reset.css';
import PeerDashboard from './pages/PeerDashboard';
import { invoke } from "@tauri-apps/api/tauri";

const App: React.FC = () => {
    const startNetwork = async () => {
        try {
          await invoke('start_network', { secretKeySeed: 1, listenAddress: '0.0.0.0', peer: 'somePeerAddress' });
          console.log('Network started');
        } catch (error) {
          console.error('Failed to start network:', error);
        }
      };

    return (
        <ConfigProvider theme={{ token: { colorPrimary: '#7091e6', colorBgBase: '#ede8f5' } }}>
            <div className="App">
                <Router>
                    <Routes>
                        <Route path="/" element={<Home />} />
                        <Route path="/contents" element={<Contents />} />
                        <Route path="/dashboard" element={<ProviderDashboard startNetwork={startNetwork} />} />
                        <Route path="/content/:id" element={<ContentDetails />} />
                        <Route path="/node-dashboard" element={<PeerDashboard />} />
                    </Routes>
                </Router>
            </div>
        </ConfigProvider>
    );
};

export default App;
