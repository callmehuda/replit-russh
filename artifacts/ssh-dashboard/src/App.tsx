import React, { useState, useEffect } from "react";
import { Terminal, Copy, Check, Server, Shield, Layers, Info, ArrowRight } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle, CardFooter } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { Input } from "@/components/ui/input";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip";

function CopyButton({ text, label }: { text: string; label?: string }) {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(text);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error("Failed to copy text: ", err);
    }
  };

  return (
    <Button
      variant="outline"
      size="sm"
      className="h-8 border-dashed bg-background/50 hover:bg-background/80 transition-all font-mono text-xs text-muted-foreground hover:text-foreground"
      onClick={handleCopy}
      data-testid={`copy-button-${text.slice(0, 10)}`}
    >
      {copied ? (
        <span className="flex items-center gap-1.5 text-primary">
          <Check className="w-3.5 h-3.5" />
          Copied
        </span>
      ) : (
        <span className="flex items-center gap-1.5">
          <Copy className="w-3.5 h-3.5" />
          {label || "Copy"}
        </span>
      )}
    </Button>
  );
}

function CodeBlock({ code, editableHost }: { code: string; editableHost?: boolean }) {
  const [host, setHost] = useState("localhost");

  const displayCode = editableHost ? code.replace("<HOST>", host) : code;

  return (
    <div className="relative group rounded-md border border-border bg-black/40 overflow-hidden mt-3 transition-colors hover:border-primary/30">
      <div className="flex items-center justify-between px-4 py-2 border-b border-border/50 bg-black/20">
        <div className="flex items-center gap-2">
          <div className="flex gap-1.5">
            <div className="w-2.5 h-2.5 rounded-full bg-red-500/20 border border-red-500/50"></div>
            <div className="w-2.5 h-2.5 rounded-full bg-yellow-500/20 border border-yellow-500/50"></div>
            <div className="w-2.5 h-2.5 rounded-full bg-green-500/20 border border-green-500/50"></div>
          </div>
          <span className="text-xs font-mono text-muted-foreground ml-2">bash</span>
        </div>
        <CopyButton text={displayCode} />
      </div>
      
      <div className="p-4 font-mono text-sm leading-relaxed overflow-x-auto whitespace-pre-wrap">
        {editableHost ? (
          <div className="flex flex-wrap items-center">
            <span className="text-blue-400">websocat</span>
            <span className="text-muted-foreground mx-2">-b</span>
            <span className="text-green-400">tcp-l:127.0.0.1:2222</span>
            <span className="text-muted-foreground mx-2">ws://</span>
            <Input 
              value={host}
              onChange={(e) => setHost(e.target.value)}
              className="h-6 w-28 px-1.5 py-0 mx-0.5 bg-primary/10 border-primary/30 text-primary font-mono text-sm text-center rounded focus-visible:ring-1 focus-visible:ring-primary/50"
              placeholder="localhost"
            />
            <span className="text-muted-foreground">:8022</span>
          </div>
        ) : (
          <div className="text-gray-300">
            <span className="text-blue-400">ssh</span>
            <span className="text-muted-foreground mx-2">-o</span>
            <span className="text-orange-300">StrictHostKeyChecking=no</span>
            <span className="text-muted-foreground mx-2">-p</span>
            <span className="text-green-400">2222</span>
            <span className="ml-2 text-gray-300">admin@localhost</span>
          </div>
        )}
      </div>
    </div>
  );
}

function Dashboard() {
  return (
    <div className="min-h-[100dvh] w-full bg-background text-foreground selection:bg-primary/20 selection:text-primary">
      <div className="fixed inset-0 pointer-events-none opacity-40 bg-[url('data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSI0IiBoZWlnaHQ9IjQiPjxyZWN0IHdpZHRoPSI0IiBoZWlnaHQ9IjQiIGZpbGw9IiNmZmYiIGZpbGwtb3BhY2l0eT0iMC4wMSIvPjwvc3ZnPg==')] z-0"></div>
      
      <main className="container max-w-4xl mx-auto px-4 py-12 relative z-10">
        
        {/* Header */}
        <header className="mb-10 flex flex-col md:flex-row md:items-end justify-between gap-6 animate-in slide-in-from-bottom-4 fade-in duration-500">
          <div>
            <div className="flex items-center gap-3 mb-3">
              <div className="p-2 rounded-lg bg-primary/10 border border-primary/20">
                <Terminal className="w-6 h-6 text-primary" />
              </div>
              <h1 className="text-3xl font-bold tracking-tight">ws-ssh-server</h1>
            </div>
            <p className="text-muted-foreground max-w-lg">
              A lightweight Rust SSH server running entirely over WebSockets. 
              No sshd required. Connect securely from anywhere.
            </p>
          </div>
          
          <div className="flex items-center gap-3 bg-secondary/30 border border-border p-3 rounded-lg backdrop-blur-sm">
            <div className="flex items-center gap-2">
              <span className="relative flex h-3 w-3">
                <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-primary opacity-75"></span>
                <span className="relative inline-flex rounded-full h-3 w-3 bg-primary"></span>
              </span>
              <span className="text-sm font-medium tracking-wide text-primary">RUNNING</span>
            </div>
            <Separator orientation="vertical" className="h-6" />
            <div className="text-xs text-muted-foreground font-mono">
              ws://127.0.0.1:8022
            </div>
          </div>
        </header>

        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          
          {/* Main Content Column */}
          <div className="lg:col-span-2 space-y-6">
            
            {/* Connection Guide */}
            <Card className="border-border/60 bg-card/50 backdrop-blur-md shadow-xl overflow-hidden animate-in slide-in-from-bottom-6 fade-in duration-700 delay-100 fill-mode-both">
              <CardHeader className="border-b border-border/40 bg-muted/20 pb-4">
                <div className="flex items-center gap-2 text-primary">
                  <ArrowRight className="w-4 h-4" />
                  <CardTitle className="text-lg">Connection Guide</CardTitle>
                </div>
                <CardDescription>
                  Follow these steps to establish a secure SSH tunnel over WebSocket.
                </CardDescription>
              </CardHeader>
              <CardContent className="p-0">
                
                <div className="p-6 border-b border-border/40">
                  <div className="flex items-start gap-4">
                    <div className="flex items-center justify-center w-6 h-6 rounded-full bg-primary/20 text-primary text-xs font-bold shrink-0 mt-0.5">
                      1
                    </div>
                    <div className="flex-1">
                      <h3 className="text-base font-semibold text-foreground mb-1">Bridge the connection</h3>
                      <p className="text-sm text-muted-foreground mb-2">
                        Use websocat to listen on a local TCP port and forward traffic to the WebSocket server.
                      </p>
                      <CodeBlock code="websocat -b tcp-l:127.0.0.1:2222 ws://<HOST>:8022" editableHost={true} />
                    </div>
                  </div>
                </div>

                <div className="p-6 bg-muted/5">
                  <div className="flex items-start gap-4">
                    <div className="flex items-center justify-center w-6 h-6 rounded-full bg-primary/20 text-primary text-xs font-bold shrink-0 mt-0.5">
                      2
                    </div>
                    <div className="flex-1">
                      <h3 className="text-base font-semibold text-foreground mb-1">Connect via SSH</h3>
                      <p className="text-sm text-muted-foreground mb-2">
                        Connect your standard SSH client to the local bridged port.
                      </p>
                      <CodeBlock code="ssh -o StrictHostKeyChecking=no -p 2222 admin@localhost" />
                    </div>
                  </div>
                </div>

              </CardContent>
            </Card>

            {/* Architecture / How it works */}
            <Card className="border-border/60 bg-card/50 backdrop-blur-md animate-in slide-in-from-bottom-8 fade-in duration-700 delay-300 fill-mode-both">
              <CardHeader>
                <div className="flex items-center gap-2">
                  <Info className="w-4 h-4 text-muted-foreground" />
                  <CardTitle className="text-base">How it works</CardTitle>
                </div>
              </CardHeader>
              <CardContent className="text-sm text-muted-foreground leading-relaxed space-y-4">
                <p>
                  This server accepts native WebSocket connections and runs the standard SSH protocol on top of the WebSocket payload payload. 
                  It does <strong className="text-foreground">not</strong> require the system `sshd` daemon to be running.
                </p>
                <div className="flex items-center justify-between gap-2 p-4 rounded-lg bg-black/20 border border-border/40 font-mono text-xs overflow-hidden">
                  <div className="flex flex-col items-center gap-2 w-24">
                    <div className="p-2 rounded bg-blue-500/10 border border-blue-500/20 text-blue-400">SSH Client</div>
                    <span className="text-muted-foreground">Port 2222</span>
                  </div>
                  <div className="flex-1 flex flex-col items-center">
                    <div className="h-px w-full bg-gradient-to-r from-transparent via-primary/50 to-transparent"></div>
                    <span className="text-[10px] text-primary py-1">websocat (TCP to WS)</span>
                  </div>
                  <div className="flex flex-col items-center gap-2 w-28">
                    <div className="p-2 rounded bg-green-500/10 border border-green-500/20 text-green-400">WS-SSH Server</div>
                    <span className="text-muted-foreground">Port 8022</span>
                  </div>
                </div>
              </CardContent>
            </Card>

          </div>

          {/* Sidebar Column */}
          <div className="space-y-6">
            
            {/* Credentials */}
            <Card className="border-border/60 bg-card/50 backdrop-blur-md animate-in slide-in-from-bottom-6 fade-in duration-700 delay-200 fill-mode-both">
              <CardHeader className="pb-4">
                <div className="flex items-center gap-2">
                  <Shield className="w-4 h-4 text-primary" />
                  <CardTitle className="text-base">Demo Credentials</CardTitle>
                </div>
              </CardHeader>
              <CardContent>
                <div className="rounded-md border border-border overflow-hidden">
                  <table className="w-full text-sm">
                    <thead>
                      <tr className="bg-muted/30 border-b border-border">
                        <th className="text-left font-medium p-3 text-muted-foreground">Username</th>
                        <th className="text-left font-medium p-3 text-muted-foreground">Password</th>
                      </tr>
                    </thead>
                    <tbody className="divide-y divide-border/50">
                      <tr className="hover:bg-muted/10 transition-colors">
                        <td className="p-3 font-mono text-foreground">admin</td>
                        <td className="p-3 font-mono text-muted-foreground">
                          <div className="flex items-center justify-between group">
                            <span>secret123</span>
                            <Button 
                              variant="ghost" 
                              size="icon" 
                              className="h-6 w-6 opacity-0 group-hover:opacity-100 transition-opacity"
                              onClick={() => navigator.clipboard.writeText("secret123")}
                            >
                              <Copy className="w-3 h-3" />
                            </Button>
                          </div>
                        </td>
                      </tr>
                      <tr className="hover:bg-muted/10 transition-colors">
                        <td className="p-3 font-mono text-foreground">guest</td>
                        <td className="p-3 font-mono text-muted-foreground">
                          <div className="flex items-center justify-between group">
                            <span>guest</span>
                            <Button 
                              variant="ghost" 
                              size="icon" 
                              className="h-6 w-6 opacity-0 group-hover:opacity-100 transition-opacity"
                              onClick={() => navigator.clipboard.writeText("guest")}
                            >
                              <Copy className="w-3 h-3" />
                            </Button>
                          </div>
                        </td>
                      </tr>
                    </tbody>
                  </table>
                </div>
              </CardContent>
            </Card>

            {/* Tech Stack */}
            <Card className="border-border/60 bg-card/50 backdrop-blur-md animate-in slide-in-from-bottom-8 fade-in duration-700 delay-400 fill-mode-both">
              <CardHeader className="pb-4">
                <div className="flex items-center gap-2">
                  <Layers className="w-4 h-4 text-primary" />
                  <CardTitle className="text-base">Stack Info</CardTitle>
                </div>
              </CardHeader>
              <CardContent>
                <div className="flex flex-wrap gap-2">
                  <Badge variant="secondary" className="bg-secondary/50 hover:bg-secondary/80 text-secondary-foreground font-mono">Rust</Badge>
                  <Badge variant="secondary" className="bg-secondary/50 hover:bg-secondary/80 text-secondary-foreground font-mono">Tokio</Badge>
                  <Badge variant="secondary" className="bg-secondary/50 hover:bg-secondary/80 text-secondary-foreground font-mono">russh 0.44</Badge>
                  <Badge variant="secondary" className="bg-secondary/50 hover:bg-secondary/80 text-secondary-foreground font-mono">tokio-tungstenite</Badge>
                  <Badge variant="secondary" className="bg-secondary/50 hover:bg-secondary/80 text-secondary-foreground font-mono">portable-pty</Badge>
                </div>
              </CardContent>
            </Card>

          </div>

        </div>
      </main>
    </div>
  );
}

function App() {
  // Adding the dark class directly on root for this specific dashboard since it demands dark mode default
  useEffect(() => {
    document.documentElement.classList.add("dark");
  }, []);

  return (
    <TooltipProvider>
      <Dashboard />
    </TooltipProvider>
  );
}

export default App;
