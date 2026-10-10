"use client";

import { useId, useState, type FormEvent } from "react";
import styles from "./waitlist-form.module.css";

// The waitlist backend is an open decision in the spec: the form sends nothing,
// and the copy says so before and after a submit.
const DEFAULT_NOTE = "Early access is not open yet. This form does not send your address anywhere.";
const SUBMITTED_NOTE = "Nothing was sent. Early access is not open yet.";

export function WaitlistForm() {
  const emailId = useId();
  const [submits, setSubmits] = useState(0);

  function onSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setSubmits((count) => count + 1);
  }

  return (
    <>
      <form id="waitlist" className={styles.form} aria-label="Early access" onSubmit={onSubmit}>
        <label className={styles.srOnly} htmlFor={emailId}>Email address</label>
        <input
          className="field t-body"
          id={emailId}
          name="email"
          type="email"
          placeholder="you@example.com"
          autoComplete="email"
          required
        />
        <button className="btn btn-primary t-body t-strong" type="submit">Get early access</button>
      </form>
      {/* The live region stays mounted; the keyed span is replaced on every submit so it announces again. */}
      <p className={`${styles.note} t-footnote`} role="status">
        <span key={submits}>{submits === 0 ? DEFAULT_NOTE : SUBMITTED_NOTE}</span>
      </p>
    </>
  );
}
