if (
    "serviceWorker" in navigator &&
    window.location.hash !== "#dev"
) {
    window.addEventListener("load", function () {
        navigator.serviceWorker.register("sw.js");
    });
}
