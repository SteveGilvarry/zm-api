import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { MonitorsWhereInput } from './monitors-where.input';
import { Type } from 'class-transformer';

@ArgsType()
export class DeleteManyMonitorsArgs {

    @Field(() => MonitorsWhereInput, {nullable:true})
    @Type(() => MonitorsWhereInput)
    where?: MonitorsWhereInput;
}
