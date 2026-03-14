import React, { useState, useEffect } from 'react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { Tabs, TabsContent, TabsList, TabsTrigger } from './components/ui/tabs.tsx';
import { Toaster } from './components/ui/sonner.tsx';

// Onboarding Components
import SystemCheck from './components/onboarding/SystemCheck.tsx';
import WalletSetup from './components/onboarding/WalletSetup.tsx';
import NodeRegistration from './components/onboarding/NodeRegistration.tsx';

// Dashboard Components
import NodeStatus from './components/dashboard/NodeStatus.tsx';
import EarningsCard from './components/dashboard/EarningsCard.tsx';
import TaskHistory from './components/dashboard/TaskHistory.tsx';

// Settings Components
import HardwareConfig from './components/settings/HardwareConfig.tsx';
import ServiceOptions from './components/settings/ServiceOptions.tsx';

// Hooks
import { useNodeStatus } from './hooks/useNodeStatus.ts';
import { useHardwareInfo } from './hooks/useHardwareInfo.ts';

import './App.css';

const queryClient = new QueryClient();

export interface AppState {
  onboardingComplete: boolean;
  currentStep: number;
  walletAddress: string | null;
  nodeId: string | null;
}

function AppContent() {
  const [appState, setAppState] = useState<AppState>({
    onboardingComplete: false,
    currentStep: 0,
    walletAddress: null,
    nodeId: null,
  });

  const { data: nodeStatus } = useNodeStatus();
  const { data: hardwareInfo } = useHardwareInfo();

  // Check if onboarding is complete on app start
  useEffect(() => {
    const checkOnboardingStatus = async () => {
      try {
        // Check if wallet exists and node is registered
        const { invoke } = await import('@tauri-apps/api/core');
        
        const walletAddress = await invoke<string>('get_wallet_address').catch(() => null);
        
        if (walletAddress) {
          setAppState(prev => ({
            ...prev,
            onboardingComplete: true,
            walletAddress,
            nodeId: nodeStatus?.node_id || null,
          }));
        }
      } catch (error) {
        console.warn('Failed to check onboarding status:', error);
      }
    };

    checkOnboardingStatus();
  }, [nodeStatus]);

  const handleOnboardingComplete = (walletAddress: string, nodeId: string) => {
    setAppState({
      onboardingComplete: true,
      currentStep: 3,
      walletAddress,
      nodeId,
    });
  };

  const resetOnboarding = () => {
    setAppState({
      onboardingComplete: false,
      currentStep: 0,
      walletAddress: null,
      nodeId: null,
    });
  };

  if (!appState.onboardingComplete) {
    return (
      <div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100">
        <div className="container mx-auto px-4 py-8">
          <div className="text-center mb-8">
            <h1 className="text-4xl font-bold text-gray-900 mb-2">Smainer Desktop</h1>
            <p className="text-lg text-gray-600">Set up your provider node in minutes</p>
          </div>

          <div className="max-w-4xl mx-auto">
            <div className="bg-white rounded-lg shadow-lg p-8">
              <div className="flex items-center justify-center mb-8">
                <div className="flex items-center space-x-8">
                  {[
                    { step: 0, title: 'System Check', icon: '🔍' },
                    { step: 1, title: 'Wallet Setup', icon: '👛' },
                    { step: 2, title: 'Node Registration', icon: '🚀' },
                  ].map(({ step, title, icon }) => (
                    <div
                      key={step}
                      className={`flex items-center space-x-2 px-4 py-2 rounded-lg ${
                        step === appState.currentStep
                          ? 'bg-blue-100 text-blue-800'
                          : step < appState.currentStep
                          ? 'bg-green-100 text-green-800'
                          : 'bg-gray-100 text-gray-600'
                      }`}
                    >
                      <span className="text-2xl">{icon}</span>
                      <span className="font-medium">{title}</span>
                    </div>
                  ))}
                </div>
              </div>

              {appState.currentStep === 0 && (
                <SystemCheck onNext={() => setAppState(prev => ({ ...prev, currentStep: 1 }))} />
              )}
              {appState.currentStep === 1 && (
                <WalletSetup
                  onNext={(address) => {
                    setAppState(prev => ({ ...prev, currentStep: 2, walletAddress: address }));
                  }}
                  onBack={() => setAppState(prev => ({ ...prev, currentStep: 0 }))}
                />
              )}
              {appState.currentStep === 2 && (
                <NodeRegistration
                  walletAddress={appState.walletAddress!}
                  hardwareInfo={hardwareInfo}
                  onComplete={handleOnboardingComplete}
                  onBack={() => setAppState(prev => ({ ...prev, currentStep: 1 }))}
                />
              )}
            </div>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-50">
      <div className="bg-white shadow-sm border-b">
        <div className="container mx-auto px-4 py-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-4">
              <h1 className="text-2xl font-bold text-gray-900">Smainer Desktop</h1>
              <div className="flex items-center space-x-2">
                <div className={`w-3 h-3 rounded-full ${nodeStatus?.is_online ? 'bg-green-500' : 'bg-red-500'}`} />
                <span className="text-sm text-gray-600">
                  {nodeStatus?.is_online ? 'Online' : 'Offline'}
                </span>
              </div>
            </div>
            <div className="flex items-center space-x-4">
              <div className="text-right">
                <div className="text-sm text-gray-600">Today's Earnings</div>
                <div className="text-lg font-bold text-green-600">
                  ${((nodeStatus?.earnings_today || 0) / 100).toFixed(2)}
                </div>
              </div>
              <button
                onClick={resetOnboarding}
                className="px-4 py-2 text-sm text-gray-600 hover:text-gray-800 border border-gray-300 rounded-lg hover:bg-gray-50"
              >
                Reset Setup
              </button>
            </div>
          </div>
        </div>
      </div>

      <div className="container mx-auto px-4 py-6">
        <Tabs defaultValue="dashboard" className="w-full">
          <TabsList className="grid w-full grid-cols-3">
            <TabsTrigger value="dashboard">Dashboard</TabsTrigger>
            <TabsTrigger value="tasks">Tasks</TabsTrigger>
            <TabsTrigger value="settings">Settings</TabsTrigger>
          </TabsList>

          <TabsContent value="dashboard" className="space-y-6">
            <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
              <div className="lg:col-span-2">
                <NodeStatus status={nodeStatus} />
              </div>
              <div>
                <EarningsCard />
              </div>
            </div>
          </TabsContent>

          <TabsContent value="tasks">
            <TaskHistory />
          </TabsContent>

          <TabsContent value="settings" className="space-y-6">
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
              <HardwareConfig />
              <ServiceOptions onReset={resetOnboarding} />
            </div>
          </TabsContent>
        </Tabs>
      </div>
    </div>
  );
}

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <AppContent />
      <Toaster />
    </QueryClientProvider>
  );
}

export default App;