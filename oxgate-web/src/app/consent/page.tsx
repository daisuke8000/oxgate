"use client";

import { Suspense, useState } from "react";
import { useSearchParams } from "next/navigation";
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
import { CheckCircle2 } from "lucide-react";

function ConsentForm() {
  const searchParams = useSearchParams();
  const consentChallenge = searchParams.get("consent_challenge");
  const [error, setError] = useState<string>("");

  const consentMutation = useMutation({
    mutationFn: (accept: boolean) =>
      apiClient.consent({
        consent_challenge: consentChallenge || "",
        accept,
        grant_scope: accept ? ["openid", "profile", "email"] : undefined,
      }),
    onSuccess: (data) => {
      window.location.href = data.redirect_to;
    },
    onError: (error: ApiError) => {
      setError(error.message || "同意処理に失敗しました");
    },
  });

  const handleAccept = () => {
    setError("");
    consentMutation.mutate(true);
  };

  const handleReject = () => {
    setError("");
    consentMutation.mutate(false);
  };

  if (!consentChallenge) {
    return (
      <Card className="w-full max-w-md" role="alert">
        <CardHeader>
          <CardTitle>エラー</CardTitle>
          <CardDescription>
            consent_challenge パラメータが必要です
          </CardDescription>
        </CardHeader>
      </Card>
    );
  }

  return (
    <Card className="w-full max-w-md">
      <CardHeader>
        <CardTitle>アクセス許可</CardTitle>
        <CardDescription>
          アプリケーションが以下の情報へのアクセスを要求しています
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        {error && <ErrorMessage message={error} />}

        <div className="space-y-3 rounded-lg border bg-muted/50 p-4">
          <div className="flex items-center gap-2">
            <CheckCircle2 className="h-5 w-5 text-primary" />
            <div>
              <p className="font-medium">基本プロフィール</p>
              <p className="text-sm text-muted-foreground">
                ユーザーID、名前
              </p>
            </div>
          </div>
          <div className="flex items-center gap-2">
            <CheckCircle2 className="h-5 w-5 text-primary" />
            <div>
              <p className="font-medium">メールアドレス</p>
              <p className="text-sm text-muted-foreground">
                登録済みのメールアドレス
              </p>
            </div>
          </div>
        </div>

        <p className="text-sm text-muted-foreground">
          この許可は後から取り消すことができます。
        </p>
      </CardContent>
      <CardFooter className="flex gap-3">
        <Button
          variant="outline"
          className="flex-1"
          onClick={handleReject}
          disabled={consentMutation.isPending}
        >
          拒否
        </Button>
        <Button
          className="flex-1"
          onClick={handleAccept}
          disabled={consentMutation.isPending}
        >
          {consentMutation.isPending ? "処理中..." : "許可"}
        </Button>
      </CardFooter>
    </Card>
  );
}

function LoadingFallback() {
  return (
    <Card className="w-full max-w-md">
      <CardHeader>
        <CardTitle>読み込み中...</CardTitle>
      </CardHeader>
    </Card>
  );
}

export default function ConsentPage() {
  return (
    <div className="flex min-h-screen items-center justify-center bg-muted/50">
      <Suspense fallback={<LoadingFallback />}>
        <ConsentForm />
      </Suspense>
    </div>
  );
}
