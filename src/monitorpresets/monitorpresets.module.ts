import { Module } from '@nestjs/common';
import { MonitorpresetsService } from './monitorpresets.service';
import { MonitorpresetsResolver } from './monitorpresets.resolver';
import { PrismaService} from '../../prisma/prisma.service';

@Module({
  providers: [PrismaService,MonitorpresetsResolver, MonitorpresetsService]
})
export class MonitorpresetsModule {}
