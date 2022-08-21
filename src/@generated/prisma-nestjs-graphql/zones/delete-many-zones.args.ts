import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ZonesWhereInput } from './zones-where.input';
import { Type } from 'class-transformer';

@ArgsType()
export class DeleteManyZonesArgs {

    @Field(() => ZonesWhereInput, {nullable:true})
    @Type(() => ZonesWhereInput)
    where?: ZonesWhereInput;
}
