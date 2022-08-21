import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { StatsUpdateInput } from './stats-update.input';
import { Type } from 'class-transformer';
import { StatsWhereUniqueInput } from './stats-where-unique.input';

@ArgsType()
export class UpdateOneStatsArgs {

    @Field(() => StatsUpdateInput, {nullable:false})
    @Type(() => StatsUpdateInput)
    data!: StatsUpdateInput;

    @Field(() => StatsWhereUniqueInput, {nullable:false})
    @Type(() => StatsWhereUniqueInput)
    where!: StatsWhereUniqueInput;
}
