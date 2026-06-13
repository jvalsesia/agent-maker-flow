import { SignIn } from "@clerk/clerk-react";

import { clerkAppearance } from "../lib/clerkAppearance";
import { AuthLayout } from "./AuthLayout";

/** Hosted Clerk sign-in, centered and themed, mounted at the public `/sign-in` route. */
export function SignInPage() {
  return (
    <AuthLayout label="Sign in">
      <SignIn routing="path" path="/sign-in" signUpUrl="/sign-up" appearance={clerkAppearance} />
    </AuthLayout>
  );
}
