import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { StatsUpdateManyMutationInput } from './stats-update-many-mutation.input';
import { Type } from 'class-transformer';
import { StatsWhereInput } from './stats-where.input';

@ArgsType()
export class UpdateManyStatsArgs {

    @Field(() => StatsUpdateManyMutationInput, {nullable:false})
    @Type(() => StatsUpdateManyMutationInput)
    data!: StatsUpdateManyMutationInput;

    @Field(() => StatsWhereInput, {nullable:true})
    @Type(() => StatsWhereInput)
    where?: StatsWhereInput;
}
