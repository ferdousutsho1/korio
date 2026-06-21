const $ = (id) => document.getElementById(id);

chrome.storage.local.get(["token", "port"]).then(({ token = "", port = 7878 }) => {
  $("token").value = token;
  $("port").value = port;
});

$("save").addEventListener("click", async () => {
  await chrome.storage.local.set({
    token: $("token").value.trim(),
    port: parseInt($("port").value, 10) || 7878,
  });
  $("msg").textContent = "Saved";
  setTimeout(() => ($("msg").textContent = ""), 1500);
});
