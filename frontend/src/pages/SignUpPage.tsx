import { SignUp } from "@clerk/clerk-react";

import { clerkAppearance } from "../lib/clerkAppearance";
import { AuthLayout } from "./AuthLayout";

/** Hosted Clerk sign-up, centered and themed, mounted at the public `/sign-up` route. */
export function SignUpPage() {
  return (
    <AuthLayout label="Sign up">
      <SignUp routing="path" path="/sign-up" signInUrl="/sign-in" appearance={clerkAppearance} />
    </AuthLayout>
  );
}
