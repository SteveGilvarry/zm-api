import { Module } from '@nestjs/common';
import { MonitorstatusService } from './monitorstatus.service';
import { MonitorstatusResolver } from './monitorstatus.resolver';
import { PrismaService} from '../../prisma/prisma.service';

@Module({
  providers: [PrismaService,MonitorstatusResolver, MonitorstatusService]
})
export class MonitorstatusModule {}
