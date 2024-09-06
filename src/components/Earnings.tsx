import React from 'react';
import { Card, Statistic, Row, Col } from 'antd';

const Earnings: React.FC = () => {
  return (
    <Row gutter={16} >
      <Col span={8}>
        <Card>
          <Statistic title="Total Earnings" value={1200} prefix="APT" />
        </Card>
      </Col>
      <Col span={8}>
        <Card>
          <Statistic title="This Month" value={200} prefix="APT" />
        </Card>
      </Col>
      <Col span={8}>
        <Card>
          <Statistic title="Last Month" value={150} prefix="APT" />
        </Card>
      </Col>
    </Row>
  );
};

export default Earnings;
