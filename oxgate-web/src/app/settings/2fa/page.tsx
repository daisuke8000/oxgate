"use client";

import { useState } from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { useMutation } from "@tanstack/react-query";
import { apiClient, ApiError } from "@/lib/api-client";
import {
  twoFactorSetupSchema,
  twoFactorVerifySchema,
  twoFactorDisableSchema,
  type TwoFactorSetupFormData,
  type TwoFactorVerifyFormData,
  type TwoFactorDisableFormData,
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
import { Shield, ShieldCheck, ShieldOff } from "lucide-react";
import Image from "next/image";

type SetupStep = "initial" | "setup" | "verify" | "enabled" | "disable";

export default function TwoFactorAuthPage() {
  const [step, setStep] = useState<SetupStep>("initial");
  const [error, setError] = useState<string>("");
  const [qrCode, setQrCode] = useState<string>("");
  const [secret, setSecret] = useState<string>("");

  const setupForm = useForm<TwoFactorSetupFormData>({
    resolver: zodResolver(twoFactorSetupSchema),
  });

  const verifyForm = useForm<TwoFactorVerifyFormData>({
    resolver: zodResolver(twoFactorVerifySchema),
  });

  const disableForm = useForm<TwoFactorDisableFormData>({
    resolver: zodResolver(twoFactorDisableSchema),
  });

  const setupMutation = useMutation({
    mutationFn: (data: TwoFactorSetupFormData) => apiClient.setup2FA(data),
    onSuccess: (data) => {
      setQrCode(data.qr_code);
      setSecret(data.secret);
      setStep("verify");
      setupForm.reset();
    },
    onError: (error: ApiError) => {
      setError(error.message || "2FAの設定に失敗しました");
    },
  });

  const verifyMutation = useMutation({
    mutationFn: (data: TwoFactorVerifyFormData) => apiClient.verify2FA(data),
    onSuccess: () => {
      setStep("enabled");
      verifyForm.reset();
    },
    onError: (error: ApiError) => {
      setError(error.message || "認証コードの検証に失敗しました");
    },
  });

  const disableMutation = useMutation({
    mutationFn: (data: TwoFactorDisableFormData) => apiClient.disable2FA(data),
    onSuccess: () => {
      setStep("initial");
      disableForm.reset();
    },
    onError: (error: ApiError) => {
      setError(error.message || "2FAの無効化に失敗しました");
    },
  });

  const handleSetup = (data: TwoFactorSetupFormData) => {
    setError("");
    setupMutation.mutate(data);
  };

  const handleVerify = (data: TwoFactorVerifyFormData) => {
    setError("");
    verifyMutation.mutate(data);
  };

  const handleDisable = (data: TwoFactorDisableFormData) => {
    setError("");
    disableMutation.mutate(data);
  };

  return (
    <div className="flex min-h-screen items-center justify-center bg-muted/50 p-4">
      <Card className="w-full max-w-md">
        {step === "initial" && (
          <>
            <CardHeader>
              <div className="mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-primary/10">
                <Shield className="h-6 w-6 text-primary" />
              </div>
              <CardTitle className="text-center">二要素認証</CardTitle>
              <CardDescription className="text-center">
                アカウントのセキュリティを強化するため、
                二要素認証を有効にすることをお勧めします
              </CardDescription>
            </CardHeader>
            <CardContent>
              <Button
                className="w-full"
                onClick={() => setStep("setup")}
              >
                2FAを有効にする
              </Button>
            </CardContent>
          </>
        )}

        {step === "setup" && (
          <>
            <CardHeader>
              <CardTitle>2FAの設定</CardTitle>
              <CardDescription>
                現在のパスワードを入力してください
              </CardDescription>
            </CardHeader>
            <CardContent>
              <form
                onSubmit={setupForm.handleSubmit(handleSetup)}
                className="space-y-4"
              >
                {error && <ErrorMessage message={error} />}

                <div className="space-y-2">
                  <Label htmlFor="password">パスワード</Label>
                  <Input
                    id="password"
                    type="password"
                    placeholder="••••••••"
                    {...setupForm.register("password")}
                    disabled={setupMutation.isPending}
                  />
                  {setupForm.formState.errors.password && (
                    <p className="text-sm text-destructive">
                      {setupForm.formState.errors.password.message}
                    </p>
                  )}
                </div>

                <div className="flex gap-3">
                  <Button
                    type="button"
                    variant="outline"
                    className="flex-1"
                    onClick={() => setStep("initial")}
                  >
                    キャンセル
                  </Button>
                  <Button
                    type="submit"
                    className="flex-1"
                    disabled={setupMutation.isPending}
                  >
                    {setupMutation.isPending ? "処理中..." : "次へ"}
                  </Button>
                </div>
              </form>
            </CardContent>
          </>
        )}

        {step === "verify" && (
          <>
            <CardHeader>
              <CardTitle>認証アプリの設定</CardTitle>
              <CardDescription>
                認証アプリでQRコードをスキャンしてください
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              {error && <ErrorMessage message={error} />}

              <div className="flex flex-col items-center gap-4">
                {qrCode && (
                  <div className="rounded-lg border bg-white p-4">
                    <Image
                      src={qrCode}
                      alt="QR Code"
                      width={200}
                      height={200}
                    />
                  </div>
                )}
                <div className="text-center">
                  <p className="text-sm text-muted-foreground mb-2">
                    QRコードを読み取れない場合は、以下のコードを手動で入力してください
                  </p>
                  <code className="rounded bg-muted px-2 py-1 text-sm font-mono">
                    {secret}
                  </code>
                </div>
              </div>

              <form
                onSubmit={verifyForm.handleSubmit(handleVerify)}
                className="space-y-4"
              >
                <div className="space-y-2">
                  <Label htmlFor="code">認証コード</Label>
                  <Input
                    id="code"
                    type="text"
                    placeholder="123456"
                    maxLength={6}
                    {...verifyForm.register("code")}
                    disabled={verifyMutation.isPending}
                  />
                  {verifyForm.formState.errors.code && (
                    <p className="text-sm text-destructive">
                      {verifyForm.formState.errors.code.message}
                    </p>
                  )}
                </div>

                <Button
                  type="submit"
                  className="w-full"
                  disabled={verifyMutation.isPending}
                >
                  {verifyMutation.isPending ? "検証中..." : "確認"}
                </Button>
              </form>
            </CardContent>
          </>
        )}

        {step === "enabled" && (
          <>
            <CardHeader>
              <div className="mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-primary/10">
                <ShieldCheck className="h-6 w-6 text-primary" />
              </div>
              <CardTitle className="text-center">2FA有効</CardTitle>
              <CardDescription className="text-center">
                二要素認証が有効になりました
              </CardDescription>
            </CardHeader>
            <CardContent>
              <Button
                variant="destructive"
                className="w-full"
                onClick={() => setStep("disable")}
              >
                2FAを無効にする
              </Button>
            </CardContent>
          </>
        )}

        {step === "disable" && (
          <>
            <CardHeader>
              <div className="mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-destructive/10">
                <ShieldOff className="h-6 w-6 text-destructive" />
              </div>
              <CardTitle className="text-center">2FAの無効化</CardTitle>
              <CardDescription className="text-center">
                パスワードと認証コードを入力してください
              </CardDescription>
            </CardHeader>
            <CardContent>
              <form
                onSubmit={disableForm.handleSubmit(handleDisable)}
                className="space-y-4"
              >
                {error && <ErrorMessage message={error} />}

                <div className="space-y-2">
                  <Label htmlFor="disable-password">パスワード</Label>
                  <Input
                    id="disable-password"
                    type="password"
                    placeholder="••••••••"
                    {...disableForm.register("password")}
                    disabled={disableMutation.isPending}
                  />
                  {disableForm.formState.errors.password && (
                    <p className="text-sm text-destructive">
                      {disableForm.formState.errors.password.message}
                    </p>
                  )}
                </div>

                <div className="space-y-2">
                  <Label htmlFor="disable-code">認証コード</Label>
                  <Input
                    id="disable-code"
                    type="text"
                    placeholder="123456"
                    maxLength={6}
                    {...disableForm.register("code")}
                    disabled={disableMutation.isPending}
                  />
                  {disableForm.formState.errors.code && (
                    <p className="text-sm text-destructive">
                      {disableForm.formState.errors.code.message}
                    </p>
                  )}
                </div>

                <div className="flex gap-3">
                  <Button
                    type="button"
                    variant="outline"
                    className="flex-1"
                    onClick={() => setStep("enabled")}
                  >
                    キャンセル
                  </Button>
                  <Button
                    type="submit"
                    variant="destructive"
                    className="flex-1"
                    disabled={disableMutation.isPending}
                  >
                    {disableMutation.isPending ? "処理中..." : "無効にする"}
                  </Button>
                </div>
              </form>
            </CardContent>
          </>
        )}
      </Card>
    </div>
  );
}
