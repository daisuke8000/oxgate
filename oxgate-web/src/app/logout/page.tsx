"use client";

import { useSearchParams } from "next/navigation";
import { useState } from "react";
import { useMutation } from "@tanstack/react-query";
import { apiClient, ApiError } from "@/lib/api-client";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { ErrorMessage } from "@/components/ui/error-message";
import { LogOut } from "lucide-react";

export default function LogoutPage() {
  const searchParams = useSearchParams();
  const logoutChallenge = searchParams.get("logout_challenge");
  const [error, setError] = useState<string>("");

  const logoutMutation = useMutation({
    mutationFn: () =>
      apiClient.logout({
        logout_challenge: logoutChallenge || "",
      }),
    onSuccess: (data) => {
      window.location.href = data.redirect_to;
    },
    onError: (error: ApiError) => {
      setError(error.message || "ログアウト処理に失敗しました");
    },
  });

  const handleLogout = () => {
    setError("");
    logoutMutation.mutate();
  };

  if (!logoutChallenge) {
    return (
      <div className="flex min-h-screen items-center justify-center">
        <Card className="w-full max-w-md" role="alert">
          <CardHeader>
            <CardTitle>エラー</CardTitle>
            <CardDescription>
              logout_challenge パラメータが必要です
            </CardDescription>
          </CardHeader>
        </Card>
      </div>
    );
  }

  return (
    <div className="flex min-h-screen items-center justify-center bg-muted/50">
      <Card className="w-full max-w-md">
        <CardHeader>
          <div className="mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-primary/10">
            <LogOut className="h-6 w-6 text-primary" />
          </div>
          <CardTitle className="text-center">ログアウト</CardTitle>
          <CardDescription className="text-center">
            ログアウトしてもよろしいですか？
          </CardDescription>
        </CardHeader>
        <CardContent>
          {error && <ErrorMessage message={error} />}
        </CardContent>
        <CardFooter>
          <Button
            className="w-full"
            onClick={handleLogout}
            disabled={logoutMutation.isPending}
          >
            {logoutMutation.isPending ? "ログアウト中..." : "ログアウト"}
          </Button>
        </CardFooter>
      </Card>
    </div>
  );
}
