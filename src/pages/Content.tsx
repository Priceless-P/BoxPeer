import React, { useEffect, useState } from 'react';
import { Layout, Input, Row, Col, Card, Button, Typography } from 'antd';
import { useNavigate } from 'react-router-dom';
import Nav from './Nav';
import Footer_ from './Footer';

const { Content } = Layout;
const { Text } = Typography;
const { Search } = Input;

interface ContentItem {
  id: number;
  title: string;
  type: string;
  size: string;
  cost: string;
  description: string;
}

const Contents: React.FC = () => {
  const [contents, setContents] = useState<ContentItem[]>([]);
  const navigate = useNavigate();

  useEffect(() => {
    const fetchContents = () => {
      const socket = new WebSocket("ws://127.0.0.1:8080");

      socket.onopen = () => {
        socket.send("get_all_contents");
      };

      socket.onmessage = (event) => {
        const data = JSON.parse(event.data);
        setContents(data);
      };

      return () => {
        socket.close();
      };
    };

    fetchContents();
  }, []);

  const handleSearch = (value: string) => {
    console.log('Search value:', value);
    // Implement search functionality if needed
  };

  const handleContentSelect = (content: ContentItem) => {
    navigate(`/content/${content.id}`, { state: { content } });
  };

  return (
    <Layout style={{ minHeight: '100vh' }}>
      <Nav />

      <Content style={{ padding: '20px 50px' }}>
        <Search placeholder="Search for content..." enterButton onSearch={handleSearch} />
      </Content>

      <Content style={{ padding: '20px 50px' }}>
        <Row gutter={[16, 16]}>
          {contents.map((item) => (
            <Col span={8} key={item.id}>
              <Card
                hoverable
                title={item.title}
                actions={[
                  <Button type="primary" onClick={() => handleContentSelect(item)}>View Details</Button>,
                ]}
              >
                {item.type.startsWith("Video") ? (
                  <video width="100%" controls>
                    <source src={`path_to_your_video/${item.id}.mp4`} type="video/mp4" />
                    Your browser does not support the video tag.
                  </video>
                ) : item.type.startsWith("Image") ? (
                  <img src={`path_to_your_images/${item.id}.jpg`} alt={item.title} width="100%" />
                ) : null}
                <Text>Type: {item.type}</Text><br />
                <Text>Size: {item.size}</Text><br />
                <Text>Cost: {item.cost}</Text>
              </Card>
            </Col>
          ))}
        </Row>
      </Content>

      <Footer_ />
    </Layout>
  );
};

export default Contents;
