// Shared by the three hero specimens: draws the app stand-in inside every
// MacBook screen, keeps it scaled to the screen, and stops the demo waitlist
// form from sending anything (the waitlist backend is an open decision).

const icon = (d) =>
  `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">${d}</svg>`;

const SCREEN = `
<div class="screen" aria-hidden="true">
  <div class="menubar"><span>&#63743;</span><b>tiny</b><span>File</span><span>Edit</span><span>View</span><span>Window</span>
    <span class="right"><span>100%</span><span>Fri 9 Oct 19:40</span></span></div>
  <div class="win">
    <div class="side">
      <div class="lights"><i></i><i></i><i></i></div>
      <div class="nav">${icon('<path d="M3 12h4l3-8 4 16 3-8h4"/>')}Processes</div>
      <div class="nav on">${icon('<path d="M4 7h16M9 7V4h6v3M6 7l1 13h10l1-13"/>')}Clean</div>
      <div class="nav">${icon('<circle cx="11" cy="11" r="6"/><path d="m20 20-4.5-4.5"/>')}Scan<span class="soon">Planned</span></div>
      <div class="free">Free on Macintosh HD<b>61.4 GB</b>of 494.4 GB<div class="bar"><i></i></div></div>
    </div>
    <div class="main">
      <div><h4>Clean</h4><div class="sub">Everything below goes to the Trash first. Nothing is deleted until you empty it.</div></div>
      <div class="sum"><div>Selected<b>18.6 GB</b></div><div>Categories<b>4 of 9</b></div><div>Skipped while app is open<b>Docker</b></div></div>
      <div class="rows">
        <div class="row"><span class="box on"></span><span>Xcode DerivedData<small>~/Library/Developer/Xcode/DerivedData</small></span><span class="risk">Safe</span><span class="size">9.8 GB</span></div>
        <div class="row"><span class="box on"></span><span>node_modules idle 30+ days<small>14 projects under ~/Code</small></span><span class="risk">Safe</span><span class="size">5.1 GB</span></div>
        <div class="row"><span class="box on"></span><span>Browser caches<small>Safari, Chrome, Arc</small></span><span class="risk">Safe</span><span class="size">2.3 GB</span></div>
        <div class="row"><span class="box on"></span><span>iOS Simulators<small>3 unavailable runtimes</small></span><span class="risk">Safe</span><span class="size">1.4 GB</span></div>
        <div class="row"><span class="box"></span><span>Downloads older than 90 days<small>Review before moving</small></span><span class="risk review">Review</span><span class="size">3.2 GB</span></div>
      </div>
      <div class="foot"><code>tiny clean --category xcode-derived --category node-modules --dry-run</code><span class="go">Move 18.6 GB to Trash</span></div>
    </div>
  </div>
</div>`;

document.querySelectorAll(".mbp-screen").forEach((el) => {
  el.insertAdjacentHTML("beforeend", SCREEN);
  const screen = el.querySelector(".screen");
  const fit = () => { screen.style.transform = `scale(${el.clientWidth / 1440})`; };
  new ResizeObserver(fit).observe(el);
  fit();
});

document.querySelectorAll("form.waitlist").forEach((form) => {
  form.addEventListener("submit", (e) => {
    e.preventDefault();
    const note = form.parentElement.querySelector(".form-note");
    if (note) note.textContent = "Specimen only: this form does not send your address anywhere yet.";
  });
});
