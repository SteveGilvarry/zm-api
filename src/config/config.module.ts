import { Module } from '@nestjs/common';
import { ConfigService } from './config.service';
import { ConfigResolver } from './config.resolver';
import { PrismaService } from '../../prisma/prisma.service';

@Module({
  providers: [PrismaService,ConfigResolver, ConfigService]
})
export class ConfigModule {}
