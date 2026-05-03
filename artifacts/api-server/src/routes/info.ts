import { Router } from "express";

const router = Router();

router.get("/info", (_req, res) => {
  const domains = process.env.REPLIT_DOMAINS ?? "";
  const primaryDomain = domains.split(",")[0]?.trim() ?? "";
  const wsUrl = primaryDomain ? `wss://${primaryDomain}/ssh` : null;

  res.json({
    wsUrl,
    domain: primaryDomain || null,
  });
});

export default router;
