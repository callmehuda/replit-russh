import { Router, type IRouter } from "express";
import healthRouter from "./health";
import infoRouter from "./info";

const router: IRouter = Router();

router.use(healthRouter);
router.use(infoRouter);

export default router;
