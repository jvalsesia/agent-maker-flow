import { SignUp } from "@clerk/clerk-react";

/** Hosted Clerk sign-up, mounted at the public `/sign-up` route. */
export function SignUpPage() {
  return (
    <main aria-label="Sign up">
      <SignUp routing="path" path="/sign-up" signInUrl="/sign-in" />
    </main>
  );
}
