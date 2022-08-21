import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { MontageLayoutsCreateInput } from './montage-layouts-create.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateOneMontageLayoutsArgs {

    @Field(() => MontageLayoutsCreateInput, {nullable:false})
    @Type(() => MontageLayoutsCreateInput)
    data!: MontageLayoutsCreateInput;
}
