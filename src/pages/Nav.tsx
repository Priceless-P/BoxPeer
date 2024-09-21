import { Menu, Layout} from 'antd';
import { Link } from 'react-router-dom';

const { Header } = Layout;

const Nav = () => {
    return (
        <Header style={{ backgroundColor: '#fff', textAlign: 'center', padding: '0' }}>
            <Menu mode="horizontal" defaultSelectedKeys={['home']} style={{ justifyContent: 'center' }}>
                <Menu.Item key="home">
                    <Link to="/">Home</Link>
                </Menu.Item>
                <Menu.Item key="features">
                    <Link to="/dashboard">Dashboard</Link>
                </Menu.Item>
                <Menu.Item key="login">
                    <Link to="/login">Login / Signup</Link>
                </Menu.Item>
                <Menu.Item key="contact">
                    <Link to="/contents">Contents</Link>
                </Menu.Item>
            </Menu>
        </Header>
    );
}

export default Nav;
