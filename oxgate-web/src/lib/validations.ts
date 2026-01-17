import { z } from "zod";

export const loginSchema = z.object({
  email: z.string().email("有効なメールアドレスを入力してください"),
  password: z.string().min(8, "パスワードは8文字以上である必要があります"),
});

export const registerSchema = z
  .object({
    email: z.string().email("有効なメールアドレスを入力してください"),
    password: z.string().min(8, "パスワードは8文字以上である必要があります"),
    confirmPassword: z.string(),
  })
  .refine((data) => data.password === data.confirmPassword, {
    message: "パスワードが一致しません",
    path: ["confirmPassword"],
  });

export const passwordResetRequestSchema = z.object({
  email: z.string().email("有効なメールアドレスを入力してください"),
});

export const passwordResetConfirmSchema = z
  .object({
    newPassword: z.string().min(8, "パスワードは8文字以上である必要があります"),
    confirmPassword: z.string(),
  })
  .refine((data) => data.newPassword === data.confirmPassword, {
    message: "パスワードが一致しません",
    path: ["confirmPassword"],
  });

export const twoFactorSetupSchema = z.object({
  password: z.string().min(1, "現在のパスワードを入力してください"),
});

export const twoFactorVerifySchema = z.object({
  code: z.string().length(6, "認証コードは6桁である必要があります"),
});

export const twoFactorDisableSchema = z.object({
  password: z.string().min(1, "現在のパスワードを入力してください"),
  code: z.string().length(6, "認証コードは6桁である必要があります"),
});

export type LoginFormData = z.infer<typeof loginSchema>;
export type RegisterFormData = z.infer<typeof registerSchema>;
export type PasswordResetRequestFormData = z.infer<typeof passwordResetRequestSchema>;
export type PasswordResetConfirmFormData = z.infer<typeof passwordResetConfirmSchema>;
export type TwoFactorSetupFormData = z.infer<typeof twoFactorSetupSchema>;
export type TwoFactorVerifyFormData = z.infer<typeof twoFactorVerifySchema>;
export type TwoFactorDisableFormData = z.infer<typeof twoFactorDisableSchema>;
