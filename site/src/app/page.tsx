import { SiteHeader } from "@/components/site-header";
import { Hero } from "@/components/hero";
import { Features } from "@/components/sections/features";
import { Pricing } from "@/components/sections/pricing";
import { DocsQuickstart } from "@/components/sections/docs-quickstart";
import { SiteFooter } from "@/components/site-footer";

export default function Home() {
  return (
    <>
      <SiteHeader />
      <main>
        <Hero />
        <Features />
        <Pricing />
        <DocsQuickstart />
      </main>
      <SiteFooter />
    </>
  );
}
