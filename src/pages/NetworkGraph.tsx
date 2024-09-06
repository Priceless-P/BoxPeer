import React, { useRef, useEffect } from 'react';
import { Network } from 'vis-network';

const NetworkGraph: React.FC = () => {
  const visRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    if (!visRef.current) return;

    const nodes = [
      { id: 1, label: 'Peer 1' },
      { id: 2, label: 'Peer 2' },
      { id: 3, label: 'Peer 3' },
    ];

    const edges = [
      { from: 1, to: 2 },
      { from: 2, to: 3 },
    ];

    const network = new Network(visRef.current, { nodes, edges }, {});

    return () => network.destroy();
  }, []);

  return <div ref={visRef} style={{ width: '300px', height: '250px' }} />;
};

export default NetworkGraph;
