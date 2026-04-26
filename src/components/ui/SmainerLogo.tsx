import React from 'react';

interface SmainerLogoProps {
  size?: number;
  className?: string;
  variant?: 'full' | 'white' | 'monochrome';
}

/**
 * Smainer Distributed Compute Logo Component
 * 
 * Modular compute blocks forming a distributed 'S' protocol.
 * Each block represents an independent compute node in the network.
 * Engineered for scalability from 24px to massive displays.
 */
export function SmainerLogo({ size = 32, className = '', variant = 'full' }: SmainerLogoProps) {
  // Color variants for different use cases
  const colors = {
    full: {
      background: '#09090B',  // Void space
      primary: '#FFFFFF',     // Core compute nodes
      terminal: '#3B82F6'     // Terminal/routing nodes
    },
    white: {
      background: 'transparent',
      primary: '#FFFFFF',
      terminal: '#FFFFFF'
    },
    monochrome: {
      background: 'transparent',
      primary: 'currentColor',
      terminal: 'currentColor'
    }
  };

  const colorSet = colors[variant];
  const scale = size / 512; // SVG is designed at 512x512

  return (
    <svg 
      width={size} 
      height={size} 
      viewBox="0 0 512 512" 
      xmlns="http://www.w3.org/2000/svg"
      className={className}
      role="img"
      aria-label="Smainer distributed compute blocks forming modular S protocol"
    >
      {/* Background container - rounded for modern feel */}
      <rect width="512" height="512" rx="96" fill={colorSet.background}/>
      
      {/* Distributed compute blocks arranged in S formation */}
      {/* Top row */}
      <rect x="200" y="104" width="112" height="48" rx="8" fill={colorSet.primary}/>
      <rect x="328" y="104" width="48" height="48" rx="8" fill={colorSet.terminal}/>
      
      {/* Second row */}
      <rect x="136" y="168" width="48" height="48" rx="8" fill={colorSet.primary}/>
      
      {/* Third row */}
      <rect x="200" y="232" width="112" height="48" rx="8" fill={colorSet.primary}/>
      
      {/* Fourth row */}
      <rect x="328" y="296" width="48" height="48" rx="8" fill={colorSet.primary}/>
      
      {/* Bottom row */}
      <rect x="136" y="360" width="48" height="48" rx="8" fill={colorSet.terminal}/>
      <rect x="200" y="360" width="112" height="48" rx="8" fill={colorSet.primary}/>
    </svg>
  );
}

export default SmainerLogo;
