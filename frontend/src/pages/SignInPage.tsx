import { SignIn } from "@clerk/clerk-react";

/** Hosted Clerk sign-in, mounted at the public `/sign-in` route. */
export function SignInPage() {
  return (
    <main aria-label="Sign in">
      <SignIn routing="path" path="/sign-in" signUpUrl="/sign-up" />
    </main>
  );
}
