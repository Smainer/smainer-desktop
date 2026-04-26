import React, { useState, useEffect } from 'react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { Tabs, TabsContent, TabsList, TabsTrigger } from './components/ui/tabs.tsx';
import { Toaster } from './components/ui/sonner.tsx';
import { SmainerLogo } from './components/ui/SmainerLogo.tsx';

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
        const { invoke } = await import('@tauri-apps/api/core');
        
        // Check if wallet exists
        const walletAddress = await invoke<string>('get_wallet_address').catch(() => null);
        
        if (walletAddress) {
          // Check if node is actually registered with the relayer
          const registrationStatus = await invoke<boolean>('check_registration_status', { walletAddress }).catch(() => false);
          
          if (registrationStatus) {
            setAppState(prev => ({
              ...prev,
              onboardingComplete: true,
              walletAddress,
              nodeId: nodeStatus?.node_id || null,
            }));
          } else {
            // Wallet exists but not registered - start from wallet setup
            setAppState(prev => ({
              ...prev,
              currentStep: 1,
              walletAddress,
            }));
          }
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
      <div className="min-h-screen bg-void">
        <div className="container mx-auto px-6 py-8">
          <div className="text-center mb-12">
            <h1 className="text-4xl font-bold text-white mb-4 tracking-tight">Smainer Desktop</h1>
            <p className="text-lg text-muted-foreground">Set up your provider node in minutes</p>
          </div>

          <div className="max-w-4xl mx-auto">
            <div className="bg-card rounded-lg border border-border p-8">
              <div className="flex items-center justify-center mb-8">
                <div className="flex items-center space-x-8">
                  {[
                    { step: 0, title: 'System Check', component: 'CheckCircle' },
                    { step: 1, title: 'Wallet Setup', component: 'Wallet' },
                    { step: 2, title: 'Node Registration', component: 'Server' },
                  ].map(({ step, title, component }) => {
                    const isActive = step === appState.currentStep;
                    const isCompleted = step < appState.currentStep;
                    const baseClasses = 'flex items-center space-x-3 px-6 py-3 rounded-lg border';
                    const stateClasses = isActive 
                      ? 'bg-primary/10 text-primary border-primary'
                      : isCompleted
                      ? 'bg-card text-card-foreground border-border'
                      : 'bg-muted/50 text-muted-foreground border-border';
                    
                    return (
                      <div key={step} className={baseClasses + ' ' + stateClasses}>
                        <div className="w-6 h-6 flex items-center justify-center">
                          {component === 'CheckCircle' && (
                            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                            </svg>
                          )}
                          {component === 'Wallet' && (
                            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 10h18M7 15h1m4 0h1m-7 4h12a3 3 0 003-3V8a3 3 0 00-3-3H6a3 3 0 00-3 3v8a3 3 0 003 3z" />
                            </svg>
                          )}
                          {component === 'Server' && (
                            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01" />
                            </svg>
                          )}
                        </div>
                        <span className="font-medium tracking-tight">{title}</span>
                      </div>
                    );
                  })}
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
    <div className="min-h-screen bg-void">
      <div className="bg-card shadow-sm border-b border-border">
        <div className="container mx-auto px-6 py-6">
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-6">
              <div className="flex items-center space-x-4">
                <SmainerLogo size={36} variant="white" />
                <h1 className="text-2xl font-bold text-white tracking-tight">Smainer Desktop</h1>
              </div>
              <div className="flex items-center space-x-3">
                <div className={'w-3 h-3 rounded-full ' + (nodeStatus?.is_online ? 'bg-primary' : 'bg-destructive')} />
                <span className="text-sm text-muted-foreground">
                  {nodeStatus?.is_online ? 'Online' : 'Offline'}
                </span>
              </div>
            </div>
            <div className="flex items-center space-x-6">
              <div className="text-right">
                <div className="text-sm text-muted-foreground">Today's Earnings</div>
                <div className="text-lg font-bold text-primary">
                  ${((nodeStatus?.earnings_today || 0) / 100).toFixed(2)}
                </div>
              </div>
              <button
                onClick={resetOnboarding}
                className="px-4 py-2 text-sm text-muted-foreground hover:text-white border border-border rounded-lg hover:bg-accent tracking-tight"
              >
                Reset Setup
              </button>
            </div>
          </div>
        </div>
      </div>

      <div className="container mx-auto px-6 py-8">
        <Tabs defaultValue="dashboard" className="w-full">
          <TabsList className="grid w-full grid-cols-3 bg-card/50 border border-border p-1 rounded-lg">
            <TabsTrigger value="dashboard" className="data-[state=active]:bg-primary data-[state=active]:text-primary-foreground">Dashboard</TabsTrigger>
            <TabsTrigger value="tasks" className="data-[state=active]:bg-primary data-[state=active]:text-primary-foreground">Tasks</TabsTrigger>
            <TabsTrigger value="settings" className="data-[state=active]:bg-primary data-[state=active]:text-primary-foreground">Settings</TabsTrigger>
          </TabsList>

          <TabsContent value="dashboard" className="space-y-8 mt-8">
            <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
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
