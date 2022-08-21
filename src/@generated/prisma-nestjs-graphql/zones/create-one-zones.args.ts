import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ZonesCreateInput } from './zones-create.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateOneZonesArgs {

    @Field(() => ZonesCreateInput, {nullable:false})
    @Type(() => ZonesCreateInput)
    data!: ZonesCreateInput;
}
