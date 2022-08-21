import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { MonitorsUpdateInput } from './monitors-update.input';
import { Type } from 'class-transformer';
import { MonitorsWhereUniqueInput } from './monitors-where-unique.input';

@ArgsType()
export class UpdateOneMonitorsArgs {

    @Field(() => MonitorsUpdateInput, {nullable:false})
    @Type(() => MonitorsUpdateInput)
    data!: MonitorsUpdateInput;

    @Field(() => MonitorsWhereUniqueInput, {nullable:false})
    @Type(() => MonitorsWhereUniqueInput)
    where!: MonitorsWhereUniqueInput;
}
