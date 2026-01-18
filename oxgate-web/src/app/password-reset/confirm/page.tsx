"use client";

import { Suspense, useState } from "react";
import { useRouter, useSearchParams } from "next/navigation";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { useMutation } from "@tanstack/react-query";
import { apiClient, ApiError } from "@/lib/api-client";
import {
  passwordResetConfirmSchema,
  type PasswordResetConfirmFormData,
} from "@/lib/validations";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { ErrorMessage } from "@/components/ui/error-message";
import { CheckCircle2 } from "lucide-react";

function PasswordResetConfirmForm() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const token = searchParams.get("token");
  const [error, setError] = useState<string>("");
  const [success, setSuccess] = useState(false);

  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<PasswordResetConfirmFormData>({
    resolver: zodResolver(passwordResetConfirmSchema),
  });

  const confirmMutation = useMutation({
    mutationFn: (data: PasswordResetConfirmFormData) =>
      apiClient.confirmPasswordReset({
        token: token || "",
        new_password: data.newPassword,
      }),
    onSuccess: () => {
      setSuccess(true);
    },
    onError: (error: ApiError) => {
      setError(error.message || "パスワードリセットに失敗しました");
    },
  });

  const onSubmit = (data: PasswordResetConfirmFormData) => {
    setError("");
    confirmMutation.mutate(data);
  };

  if (!token) {
    return (
      <Card className="w-full max-w-md" role="alert">
        <CardHeader>
          <CardTitle>エラー</CardTitle>
          <CardDescription>
            token パラメータが必要です
          </CardDescription>
        </CardHeader>
      </Card>
    );
  }

  if (success) {
    return (
      <Card className="w-full max-w-md">
        <CardHeader>
          <div className="mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-primary/10">
            <CheckCircle2 className="h-6 w-6 text-primary" />
          </div>
          <CardTitle className="text-center">パスワード変更完了</CardTitle>
          <CardDescription className="text-center">
            パスワードが正常に変更されました
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Button
            className="w-full"
            onClick={() => router.push("/login")}
          >
            ログインページへ
          </Button>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card className="w-full max-w-md">
      <CardHeader>
        <CardTitle>新しいパスワードの設定</CardTitle>
        <CardDescription>
          新しいパスワードを入力してください
        </CardDescription>
      </CardHeader>
      <CardContent>
        <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
          {error && <ErrorMessage message={error} />}

          <div className="space-y-2">
            <Label htmlFor="newPassword">新しいパスワード</Label>
            <Input
              id="newPassword"
              type="password"
              placeholder="••••••••"
              {...register("newPassword")}
              disabled={confirmMutation.isPending}
            />
            {errors.newPassword && (
              <p className="text-sm text-destructive">
                {errors.newPassword.message}
              </p>
            )}
          </div>

          <div className="space-y-2">
            <Label htmlFor="confirmPassword">パスワード（確認）</Label>
            <Input
              id="confirmPassword"
              type="password"
              placeholder="••••••••"
              {...register("confirmPassword")}
              disabled={confirmMutation.isPending}
            />
            {errors.confirmPassword && (
              <p className="text-sm text-destructive">
                {errors.confirmPassword.message}
              </p>
            )}
          </div>

          <Button
            type="submit"
            className="w-full"
            disabled={confirmMutation.isPending}
          >
            {confirmMutation.isPending ? "変更中..." : "パスワードを変更"}
          </Button>
        </form>
      </CardContent>
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

export default function PasswordResetConfirmPage() {
  return (
    <div className="flex min-h-screen items-center justify-center bg-muted/50">
      <Suspense fallback={<LoadingFallback />}>
        <PasswordResetConfirmForm />
      </Suspense>
    </div>
  );
}
