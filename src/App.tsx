import React from 'react';
import { ConfigProvider, Layout } from 'antd';
import { BrowserRouter as Router, Route, Routes } from 'react-router-dom';
import ProviderDashboard from './pages/ProviderDashboard';
import 'antd/dist/reset.css';
import CallbackPage from "./pages/CallbackPage";
import LoginPage from "./pages/LoginPage";


const App: React.FC = () => {

    const theme = {
        token: {
            colorPrimary: '#AD49E1',
            colorBgBase: '#EBD3F8',
            colorTextBase: '#2E073F',
            colorTextSecondary: '#7A1CAC',
        },
    };
    return (
        <ConfigProvider theme={theme}>
            <Layout>
                <div className="App">
                    <Router>
                        <Routes>
                            <Route path="/" element={<LoginPage />} />
                            <Route path="/callback" element={<CallbackPage />} />
                            <Route path="/dashboard" element={<ProviderDashboard />} />
                        </Routes>
                    </Router>
                </div>
            </Layout>
        </ConfigProvider>
    );
};

export default App;
