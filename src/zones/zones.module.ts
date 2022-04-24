import { Module } from '@nestjs/common';
import { ZonesService } from './zones.service';
import { ZonesResolver } from './zones.resolver';
import { PrismaService} from '../../prisma/prisma.service';

@Module({
  providers: [PrismaService, ZonesResolver, ZonesService]
})
export class ZonesModule {}
